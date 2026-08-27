//! Plotting helpers for [`filterkit`].
//!
//! Provides Bode, magnitude, impulse, and step-response plots. Use
//! `save` to write a file or `show` to open a temporary SVG.
//!
//! # Example
//!
//! ```no_run
//! use filterkit::design::BiquadLowpassSpec;
//! use filterkit_plot::BodePlot;
//!
//! let coeffs = BiquadLowpassSpec { f0: 2_000.0 / 48_000.0, q: 0.707 }
//!     .design()
//!     .unwrap();
//!
//! BodePlot::new(coeffs)
//!     .sample_rate(48_000.0)
//!     .title("2 kHz biquad lowpass")
//!     .show()
//!     .unwrap();
//! ```
//!
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use filterkit::response::{
    self, group_delay, impulse_response, logspace, magnitude_db_sweep, phase_unwrapped_sweep,
    step_response, FrequencyResponse,
};
use filterkit::traits::{Reset, SampleProcessor};
use plotters::coord::Shift;
use plotters::prelude::*;

const PRIMARY: RGBColor = RGBColor(31, 119, 180);
const SECONDARY: RGBColor = RGBColor(214, 39, 40);
const TERTIARY: RGBColor = RGBColor(44, 160, 44);

const GRID_BOLD: RGBColor = RGBColor(215, 215, 215);
const GRID_LIGHT: RGBColor = RGBColor(235, 235, 235);
const AXIS: RGBColor = RGBColor(70, 70, 70);
const TITLE: RGBColor = RGBColor(40, 40, 40);
const BASELINE: RGBColor = RGBColor(180, 180, 180);

const LINE_W: u32 = 3;
const FONT_TITLE: i32 = 32;
const FONT_DESC: i32 = 22;
const FONT_TICK: i32 = 18;
const PANE_MARGIN: i32 = 28;
const X_LABEL_AREA: i32 = 60;
const Y_LABEL_AREA: i32 = 96;

const MAX_DOT_MARKERS: usize = 96;

/// Errors that can come out of a `.save(...)` or `.show(...)` call.
#[derive(Debug)]
pub enum PlotError {
    /// File extension was not recognised (only `.png` and `.svg` are
    /// supported out of the box).
    UnknownExtension(String),
    /// Bubbled-up drawing or I/O error from plotters.
    Drawing(Box<dyn Error + Send + Sync>),
    /// `.show()` failed to launch the system viewer. The string is the
    /// command we tried to run.
    Open(String, std::io::Error),
}

impl std::fmt::Display for PlotError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownExtension(ext) => {
                write!(
                    f,
                    "unknown plot file extension '{ext}' (expected png or svg)"
                )
            }
            Self::Drawing(e) => write!(f, "plot drawing error: {e}"),
            Self::Open(cmd, e) => write!(f, "failed to launch viewer ('{cmd}'): {e}"),
        }
    }
}

impl Error for PlotError {}

fn temp_path(stem: &str, ext: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let nonce = nanos ^ (std::process::id() as u64);
    std::env::temp_dir().join(format!("filterkit-plot-{stem}-{nonce:x}.{ext}"))
}

fn open_path(path: &Path) -> Result<(), PlotError> {
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    Command::new(cmd)
        .arg(path)
        .status()
        .map(|_| ())
        .map_err(|e| PlotError::Open(cmd.to_string(), e))
}

fn show_via_save<F>(stem: &str, save: F) -> Result<(), PlotError>
where
    F: FnOnce(&Path) -> Result<(), PlotError>,
{
    let path = temp_path(stem, "svg");
    save(&path)?;
    open_path(&path)
}

/// Dispatch drawing to the backend selected by the file extension.
macro_rules! with_render_dispatch {
    ($path:expr, $size:expr, |$root:ident| $call:expr) => {{
        let path: &::std::path::Path = $path;
        let size: (u32, u32) = $size;
        match Backend::from_path(path)? {
            Backend::Png => {
                let $root = BitMapBackend::new(path, size).into_drawing_area();
                $root
                    .fill(&WHITE)
                    .map_err(|e| PlotError::Drawing(Box::new(e)))?;
                let $root = &$root;
                $call.map_err(PlotError::Drawing)?;
                $root
                    .present()
                    .map_err(|e| PlotError::Drawing(Box::new(e)))?;
            }
            Backend::Svg => {
                let $root = SVGBackend::new(path, size).into_drawing_area();
                $root
                    .fill(&WHITE)
                    .map_err(|e| PlotError::Drawing(Box::new(e)))?;
                let $root = &$root;
                $call.map_err(PlotError::Drawing)?;
                $root
                    .present()
                    .map_err(|e| PlotError::Drawing(Box::new(e)))?;
            }
        }
        Ok::<(), PlotError>(())
    }};
}

enum Backend {
    Png,
    Svg,
}

impl Backend {
    fn from_path(path: &Path) -> Result<Self, PlotError> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "bmp" => Ok(Self::Png),
            "svg" => Ok(Self::Svg),
            other => Err(PlotError::UnknownExtension(other.to_string())),
        }
    }
}

/// Builder for a two-pane Bode plot (magnitude in dB, phase in degrees,
/// both on a logarithmic frequency axis).
#[derive(Debug)]
pub struct BodePlot<R> {
    responder: R,
    sample_rate: f64,
    freq_range: Option<(f64, f64)>,
    n_points: usize,
    title: Option<String>,
    size: Option<(u32, u32)>,
    show_group_delay: bool,
}

impl<R: FrequencyResponse> BodePlot<R> {
    /// Start a Bode-plot builder for `r`.
    ///
    /// `r` can be a coefficient block (e.g. `BiquadCoeffs`), an SOS
    /// cascade, or any custom type implementing [`FrequencyResponse`].
    /// A reference is also accepted.
    pub fn new(r: R) -> Self {
        Self {
            responder: r,
            sample_rate: 1.0,
            freq_range: None,
            n_points: 1024,
            title: None,
            size: None,
            show_group_delay: false,
        }
    }

    pub fn sample_rate(mut self, fs: f64) -> Self {
        self.sample_rate = fs;
        self
    }
    pub fn freq_range(mut self, lo: f64, hi: f64) -> Self {
        self.freq_range = Some((lo, hi));
        self
    }
    pub fn n_points(mut self, n: usize) -> Self {
        self.n_points = n;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.size = Some((w, h));
        self
    }
    pub fn with_group_delay(mut self, yes: bool) -> Self {
        self.show_group_delay = yes;
        self
    }

    /// Render the plot. File extension picks the backend (`.png` /
    /// `.svg`).
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), PlotError> {
        let path = path.as_ref();
        let fs = self.sample_rate;
        let (lo, hi) = self
            .freq_range
            .unwrap_or_else(|| (1e-3 * fs / 2.0, 0.5 * fs));
        let freqs_axis = logspace(lo, hi, self.n_points);
        let freqs_norm: Vec<f64> = freqs_axis.iter().map(|&f| f / fs).collect();
        let mag_db = magnitude_db_sweep(&self.responder, &freqs_norm);
        let phase_deg: Vec<f64> = phase_unwrapped_sweep(&self.responder, &freqs_norm)
            .into_iter()
            .map(|p| p * 180.0 / std::f64::consts::PI)
            .collect();
        let group = if self.show_group_delay {
            Some(group_delay(&self.responder, &freqs_norm))
        } else {
            None
        };
        let title = self.title.unwrap_or_else(|| "Bode plot".to_string());
        let size = self.size.unwrap_or(if self.show_group_delay {
            (1600, 1200)
        } else {
            (1600, 900)
        });
        let x_desc = if fs == 1.0 {
            "f (cycles/sample)"
        } else {
            "f (Hz)"
        };

        let plot = BodeRenderData {
            title: &title,
            x_desc,
            lo,
            hi,
            freqs_axis: &freqs_axis,
            mag_db: &mag_db,
            phase_deg: &phase_deg,
            group: group.as_deref(),
        };

        with_render_dispatch!(path, size, |root| draw_bode(root, &plot))
    }

    /// Render the plot to a temp SVG and open it in the system viewer
    /// (Preview on macOS, default browser on Linux, etc.).
    pub fn show(self) -> Result<(), PlotError> {
        show_via_save("bode", |p| self.save(p))
    }
}

struct BodeRenderData<'a> {
    title: &'a str,
    x_desc: &'a str,
    lo: f64,
    hi: f64,
    freqs_axis: &'a [f64],
    mag_db: &'a [f64],
    phase_deg: &'a [f64],
    group: Option<&'a [f64]>,
}

fn draw_bode<DB>(
    root: &DrawingArea<DB, Shift>,
    plot: &BodeRenderData<'_>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let panes = if plot.group.is_some() { 3 } else { 2 };
    let areas = root.split_evenly((panes, 1));

    let (mag_lo, mag_hi) = pad_range_min_max(plot.mag_db, 6.0);
    let mut mag_chart = ChartBuilder::on(&areas[0])
        .caption(plot.title, ("sans-serif", FONT_TITLE, &TITLE))
        .margin(PANE_MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d((plot.lo..plot.hi).log_scale(), mag_lo..mag_hi)?;
    configure_styled_mesh(&mut mag_chart, plot.x_desc, "|H| (dB)")?;
    mag_chart.draw_series(LineSeries::new(
        plot.freqs_axis
            .iter()
            .copied()
            .zip(plot.mag_db.iter().copied()),
        PRIMARY.stroke_width(LINE_W),
    ))?;

    let (ph_lo, ph_hi) = pad_range_min_max(plot.phase_deg, 15.0);
    let mut ph_chart = ChartBuilder::on(&areas[1])
        .margin(PANE_MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d((plot.lo..plot.hi).log_scale(), ph_lo..ph_hi)?;
    configure_styled_mesh(&mut ph_chart, plot.x_desc, "phase (deg)")?;
    ph_chart.draw_series(LineSeries::new(
        plot.freqs_axis
            .iter()
            .copied()
            .zip(plot.phase_deg.iter().copied()),
        SECONDARY.stroke_width(LINE_W),
    ))?;

    if let Some(gd) = plot.group {
        let (g_lo, g_hi) = pad_range_min_max(gd, 0.5);
        let mut g_chart = ChartBuilder::on(&areas[2])
            .margin(PANE_MARGIN)
            .x_label_area_size(X_LABEL_AREA)
            .y_label_area_size(Y_LABEL_AREA)
            .build_cartesian_2d((plot.lo..plot.hi).log_scale(), g_lo..g_hi)?;
        configure_styled_mesh(&mut g_chart, plot.x_desc, "group delay (samples)")?;
        g_chart.draw_series(LineSeries::new(
            plot.freqs_axis.iter().copied().zip(gd.iter().copied()),
            TERTIARY.stroke_width(LINE_W),
        ))?;
    }
    Ok(())
}

/// Builder for a single-pane magnitude (dB) plot.
#[derive(Debug)]
pub struct MagnitudePlot<R> {
    responder: R,
    sample_rate: f64,
    freq_range: Option<(f64, f64)>,
    n_points: usize,
    title: Option<String>,
    size: (u32, u32),
    log_x: bool,
}

impl<R: FrequencyResponse> MagnitudePlot<R> {
    /// Start a magnitude-only plot builder for `r`.
    pub fn new(r: R) -> Self {
        Self {
            responder: r,
            sample_rate: 1.0,
            freq_range: None,
            n_points: 1024,
            title: None,
            size: (1600, 500),
            log_x: true,
        }
    }

    pub fn sample_rate(mut self, fs: f64) -> Self {
        self.sample_rate = fs;
        self
    }
    pub fn freq_range(mut self, lo: f64, hi: f64) -> Self {
        self.freq_range = Some((lo, hi));
        self
    }
    pub fn n_points(mut self, n: usize) -> Self {
        self.n_points = n;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.size = (w, h);
        self
    }
    pub fn linear_x(mut self) -> Self {
        self.log_x = false;
        self
    }
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), PlotError> {
        let path = path.as_ref();
        let fs = self.sample_rate;
        let (lo, hi) = self
            .freq_range
            .unwrap_or_else(|| (1e-3 * fs / 2.0, 0.5 * fs));
        let freqs_axis = if self.log_x {
            logspace(lo, hi, self.n_points)
        } else {
            response::linspace(lo, hi, self.n_points)
        };
        let freqs_norm: Vec<f64> = freqs_axis.iter().map(|&f| f / fs).collect();
        let mag_db = magnitude_db_sweep(&self.responder, &freqs_norm);
        let title = self.title.unwrap_or_else(|| "Magnitude".to_string());
        let size = self.size;
        let log_x = self.log_x;
        let x_desc = if fs == 1.0 {
            "f (cycles/sample)"
        } else {
            "f (Hz)"
        };

        let plot = MagnitudeRenderData {
            title: &title,
            x_desc,
            lo,
            hi,
            log_x,
            freqs_axis: &freqs_axis,
            mag_db: &mag_db,
        };

        with_render_dispatch!(path, size, |root| draw_magnitude(root, &plot))
    }

    /// Render to a temp SVG and open it in the system viewer.
    pub fn show(self) -> Result<(), PlotError> {
        show_via_save("magnitude", |p| self.save(p))
    }
}

struct MagnitudeRenderData<'a> {
    title: &'a str,
    x_desc: &'a str,
    lo: f64,
    hi: f64,
    log_x: bool,
    freqs_axis: &'a [f64],
    mag_db: &'a [f64],
}

fn draw_magnitude<DB>(
    root: &DrawingArea<DB, Shift>,
    plot: &MagnitudeRenderData<'_>,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let (y_lo, y_hi) = pad_range_min_max(plot.mag_db, 6.0);
    if plot.log_x {
        let mut chart = ChartBuilder::on(root)
            .caption(plot.title, ("sans-serif", FONT_TITLE, &TITLE))
            .margin(PANE_MARGIN)
            .x_label_area_size(X_LABEL_AREA)
            .y_label_area_size(Y_LABEL_AREA)
            .build_cartesian_2d((plot.lo..plot.hi).log_scale(), y_lo..y_hi)?;
        configure_styled_mesh(&mut chart, plot.x_desc, "|H| (dB)")?;
        chart.draw_series(LineSeries::new(
            plot.freqs_axis
                .iter()
                .copied()
                .zip(plot.mag_db.iter().copied()),
            PRIMARY.stroke_width(LINE_W),
        ))?;
    } else {
        let mut chart = ChartBuilder::on(root)
            .caption(plot.title, ("sans-serif", FONT_TITLE, &TITLE))
            .margin(PANE_MARGIN)
            .x_label_area_size(X_LABEL_AREA)
            .y_label_area_size(Y_LABEL_AREA)
            .build_cartesian_2d(plot.lo..plot.hi, y_lo..y_hi)?;
        configure_styled_mesh(&mut chart, plot.x_desc, "|H| (dB)")?;
        chart.draw_series(LineSeries::new(
            plot.freqs_axis
                .iter()
                .copied()
                .zip(plot.mag_db.iter().copied()),
            PRIMARY.stroke_width(LINE_W),
        ))?;
    }
    Ok(())
}

/// Builder for an impulse-response plot.
#[derive(Debug)]
pub struct ImpulsePlot<'p, P> {
    processor: &'p mut P,
    n: usize,
    title: Option<String>,
    size: (u32, u32),
}

impl<'p, P> ImpulsePlot<'p, P>
where
    P: SampleProcessor<f64, Output = f64> + Reset,
{
    /// Start an impulse-response plot for `p`. `p` is reset before
    /// sampling.
    pub fn new(p: &'p mut P) -> Self {
        Self {
            processor: p,
            n: 64,
            title: None,
            size: (1600, 500),
        }
    }

    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.size = (w, h);
        self
    }
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), PlotError> {
        let h = impulse_response(self.processor, self.n);
        let title = self.title.unwrap_or_else(|| "Impulse response".to_string());
        save_samples(
            path.as_ref(),
            self.size,
            &title,
            "n (samples)",
            "h[n]",
            &h,
            PRIMARY,
        )
    }

    /// Render to a temp SVG and open it in the system viewer.
    pub fn show(self) -> Result<(), PlotError> {
        show_via_save("impulse", |p| self.save(p))
    }
}

/// Builder for a step-response plot.
#[derive(Debug)]
pub struct StepPlot<'p, P> {
    processor: &'p mut P,
    n: usize,
    title: Option<String>,
    size: (u32, u32),
}

impl<'p, P> StepPlot<'p, P>
where
    P: SampleProcessor<f64, Output = f64> + Reset,
{
    /// Start a step-response plot for `p`. Resets `p` before sampling.
    pub fn new(p: &'p mut P) -> Self {
        Self {
            processor: p,
            n: 128,
            title: None,
            size: (1600, 500),
        }
    }

    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.size = (w, h);
        self
    }
    pub fn save(self, path: impl AsRef<Path>) -> Result<(), PlotError> {
        let s = step_response(self.processor, self.n);
        let title = self.title.unwrap_or_else(|| "Step response".to_string());
        save_samples(
            path.as_ref(),
            self.size,
            &title,
            "n (samples)",
            "y[n]",
            &s,
            SECONDARY,
        )
    }

    /// Render to a temp SVG and open it in the system viewer.
    pub fn show(self) -> Result<(), PlotError> {
        show_via_save("step", |p| self.save(p))
    }
}

fn save_samples(
    path: &Path,
    size: (u32, u32),
    title: &str,
    x_desc: &str,
    y_desc: &str,
    samples: &[f64],
    color: RGBColor,
) -> Result<(), PlotError> {
    let title = title.to_string();
    let x_desc = x_desc.to_string();
    let y_desc = y_desc.to_string();
    let samples = samples.to_vec();
    with_render_dispatch!(path, size, |root| draw_samples(
        root, &title, &x_desc, &y_desc, &samples, color,
    ))
}

fn draw_samples<DB>(
    root: &DrawingArea<DB, Shift>,
    title: &str,
    x_desc: &str,
    y_desc: &str,
    samples: &[f64],
    color: RGBColor,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let n = samples.len();
    let (y_lo, y_hi) = pad_range_min_max(samples, 0.1);
    let x_hi = n.saturating_sub(1).max(1) as f64;
    let mut chart = ChartBuilder::on(root)
        .caption(title, ("sans-serif", FONT_TITLE, &TITLE))
        .margin(PANE_MARGIN)
        .x_label_area_size(X_LABEL_AREA)
        .y_label_area_size(Y_LABEL_AREA)
        .build_cartesian_2d(0f64..x_hi, y_lo..y_hi)?;
    configure_styled_mesh(&mut chart, x_desc, y_desc)?;

    // Draw the baseline first so the series stays on top.
    if y_lo < 0.0 && y_hi > 0.0 {
        chart.draw_series(LineSeries::new(
            [(0.0, 0.0), (x_hi, 0.0)],
            BASELINE.stroke_width(1),
        ))?;
    }

    chart.draw_series(LineSeries::new(
        samples.iter().enumerate().map(|(i, &y)| (i as f64, y)),
        color.stroke_width(LINE_W),
    ))?;
    // Markers help for short responses and become noise on long ones.
    if n <= MAX_DOT_MARKERS {
        chart.draw_series(
            samples
                .iter()
                .enumerate()
                .map(|(i, &y)| Circle::new((i as f64, y), 3, color.filled())),
        )?;
    }
    Ok(())
}

/// Configure shared chart styling and axis labels.
fn configure_styled_mesh<DB, X, Y>(
    chart: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    x_desc: &str,
    y_desc: &str,
) -> Result<(), Box<dyn Error + Send + Sync>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
    X: plotters::coord::ranged1d::Ranged<ValueType = f64>
        + plotters::coord::ranged1d::ValueFormatter<f64>,
    Y: plotters::coord::ranged1d::Ranged<ValueType = f64>
        + plotters::coord::ranged1d::ValueFormatter<f64>,
{
    chart
        .configure_mesh()
        .light_line_style(GRID_LIGHT)
        .bold_line_style(GRID_BOLD)
        .axis_style(AXIS.stroke_width(1))
        .label_style(("sans-serif", FONT_TICK, &AXIS))
        .x_desc(x_desc)
        .y_desc(y_desc)
        .axis_desc_style(("sans-serif", FONT_DESC, &AXIS))
        .x_label_formatter(&format_tick)
        .y_label_formatter(&format_tick)
        .draw()?;
    Ok(())
}

/// Format tick labels without trailing zeros or `-0.0`.
fn format_tick(v: &f64) -> String {
    let v = *v;
    if v.abs() < 1e-10 {
        return "0".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    if v.abs() < 1.0 {
        format!("{v:.3}")
    } else if v.abs() < 100.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.1}")
    }
}

/// Return a padded range for the finite values in `samples`.
///
/// Non-finite values are ignored.
fn pad_range_min_max(samples: &[f64], min_pad: f64) -> (f64, f64) {
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for &s in samples {
        if s.is_finite() {
            if s < lo {
                lo = s;
            }
            if s > hi {
                hi = s;
            }
        }
    }
    if lo == f64::INFINITY || hi == f64::NEG_INFINITY {
        return (-1.0, 1.0);
    }
    let span = hi - lo;
    let pad = (span * 0.05).max(min_pad);
    (lo - pad, hi + pad)
}
