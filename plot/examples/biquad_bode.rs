//! Bode plot for the 2 kHz biquad lowpass used in
//! `filterkit/examples/biquad_lowpass.rs`, plus its impulse and step
//! response.
//!
//! Run with: `cargo run -p filterkit-plot --example biquad_bode`
//!
//! Writes `biquad_bode.png`, `biquad_impulse.png`, `biquad_step.png`
//! into the workspace root.

use filterkit::design::BiquadLowpassSpec;
use filterkit::processors::Biquad;
use filterkit_plot::{bode, impulse, step};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fs = 48_000.0_f64;
    let coeffs = BiquadLowpassSpec { f0: 2_000.0 / fs, q: 0.707 }
        .design()
        .expect("biquad design");

    bode(coeffs)
        .sample_rate(fs)
        .title("2 kHz biquad lowpass (Q = 0.707)")
        .with_group_delay(true)
        .save("biquad_bode.svg")?;

    let mut filter = Biquad::new(coeffs);
    impulse(&mut filter)
        .n(1000)
        .title("Biquad lowpass impulse response")
        .save("biquad_impulse.svg")?;

    step(&mut filter)
        .n(1000)
        .title("Biquad lowpass step response")
        .save("biquad_step.svg")?;

    println!("wrote biquad_bode.png, biquad_impulse.png, biquad_step.png");
    Ok(())
}
