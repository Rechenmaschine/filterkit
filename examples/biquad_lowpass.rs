//! Filter a sum-of-sines signal through a 2nd-order lowpass biquad and
//! print the result.
//!
//! Run with: `cargo run --example biquad_lowpass`

use filterkit::design::BiquadLowpassSpec;
use filterkit::processors::Biquad;
use filterkit::SampleProcessor;

fn main() {
    let fs = 48_000.0_f64;
    let f_signal = 1_000.0;
    let f_noise = 10_000.0;
    let n = 256;

    let coeffs = BiquadLowpassSpec {
        f0: 2_000.0 / fs,
        q: 0.707,
    }
    .design()
    .expect("biquad design");

    let mut filter = Biquad::new(coeffs);

    println!("# n  raw  filtered");
    for i in 0..n {
        let t = i as f64 / fs;
        let x = (2.0 * core::f64::consts::PI * f_signal * t).sin()
            + 0.5 * (2.0 * core::f64::consts::PI * f_noise * t).sin();
        let y = filter.process_sample(x);
        println!("{i:4}  {x:+.4}  {y:+.4}");
    }
}
