//! Resample a 48 kHz sinusoid to 44.1 kHz with a polyphase resampler.
//!
//! Run with: `cargo run --example polyphase_resample`

use filterkit::design::{Window, WindowedSincLowpassSpec};
use filterkit::stream::PolyphaseResampler;
use filterkit::StreamProcessor;

fn main() {
    // 48 kHz -> 44.1 kHz: ratio = 147 / 160 in lowest terms.
    // For demo purposes use the smaller ratio 147 / 160 with a single
    // windowed-sinc prototype.
    let up = 147_usize;
    let down = 160_usize;

    // Combined rate: design lowpass at min(fs_in, fs_out)/2 in the
    // up-sampled domain.
    let cutoff = 0.5 / (up.max(down) as f64);
    // Long-ish prototype to keep aliasing low.
    let proto = WindowedSincLowpassSpec {
        cutoff,
        window: Window::Blackman,
    }
    .design::<321>()
    .expect("prototype design");

    // Scale prototype gain by `up` so the polyphase decomposition
    // preserves passband level.
    let taps: Vec<f64> = proto.b.iter().map(|t| t * up as f64).collect();
    let mut resampler = PolyphaseResampler::new(&taps, up, down);

    let fs_in = 48_000.0;
    let f_signal = 1_000.0;
    let n_in = 4_096_usize;
    let xs: Vec<f64> = (0..n_in)
        .map(|i| (2.0 * core::f64::consts::PI * f_signal * i as f64 / fs_in).sin())
        .collect();

    // Output buffer: we expect roughly n_in * up / down output samples.
    let mut ys = vec![0.0; n_in * up / down + 16];
    let status = resampler.process_stream(&xs, &mut ys);

    println!(
        "Resampled {} input samples -> {} output samples (ratio {}/{})",
        status.consumed, status.produced, up, down
    );

    println!("First 10 output samples:");
    for (i, y) in ys.iter().take(10).enumerate() {
        println!("  {i:3}: {y:+.6}");
    }
}
