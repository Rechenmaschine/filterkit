//! Bode plot for the 2 kHz biquad lowpass used in
//! `filterkit/examples/biquad_lowpass.rs`, plus its impulse and step
//! response.
//!
//! Run with: `cargo run -p filterkit-plot --example biquad_bode`
//!
//! Writes `biquad_bode.png`, `biquad_impulse.png`, `biquad_step.png`
//! into the workspace root. To open the same plots in your system
//! viewer instead, swap `.save("…")` for `.show()`.

use filterkit::design::BiquadLowpassSpec;
use filterkit::processors::Biquad;
use filterkit_plot::{BodePlot, ImpulsePlot, StepPlot};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = 48_000.0_f64;
    let coeffs = BiquadLowpassSpec { f0: 2_000.0 / fs, q: 0.707 }
        .design()
        .expect("biquad design");

    BodePlot::new(coeffs)
        .sample_rate(fs)
        .title("2 kHz biquad lowpass (Q = 0.707)")
        .with_group_delay(true)
        .show()?;
        //.save("biquad_bode.png")?;

    let mut filter = Biquad::new(coeffs);
    ImpulsePlot::new(&mut filter)
        .n(96)
        .title("Biquad lowpass impulse response")
        .save("biquad_impulse.png")?;

    StepPlot::new(&mut filter)
        .n(192)
        .title("Biquad lowpass step response")
        .save("biquad_step.png")?;

    println!("wrote biquad_bode.png, biquad_impulse.png, biquad_step.png");
    Ok(())
}
