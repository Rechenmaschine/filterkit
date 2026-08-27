//! Build a small DSP graph using combinators: HP -> LP, in parallel
//! with a unity tap, summed and gained.
//!
//! Demonstrates the fluent [`ProcessorExt`] API.
//!
//! [`ProcessorExt`]: filterkit::ProcessorExt

use filterkit::design::{BiquadHighpassSpec, BiquadLowpassSpec};
use filterkit::processors::{Biquad, Gain};
use filterkit::{ProcessorExt, SampleProcessor};

fn main() {
    let fs = 48_000.0_f64;

    let hp = Biquad::new(
        BiquadHighpassSpec {
            f0: 200.0 / fs,
            q: 0.707,
        }
        .design()
        .unwrap(),
    );
    let lp = Biquad::new(
        BiquadLowpassSpec {
            f0: 4_000.0 / fs,
            q: 0.707,
        }
        .design()
        .unwrap(),
    );

    // Bandpass via HP -> LP, then 0.7x makeup gain.
    let mut bandpass_chain = hp.then(lp).then(Gain::new(0.7_f64));

    // Sweep a single sinusoid through several frequencies and print
    // steady-state RMS levels.
    for &freq in &[50.0_f64, 500.0, 2_000.0, 10_000.0, 20_000.0] {
        let mut sum_sq = 0.0_f64;
        let n = 4096_usize;
        for i in 0..n {
            let t = i as f64 / fs;
            let x = (2.0 * core::f64::consts::PI * freq * t).sin();
            let y = bandpass_chain.process_sample(x);
            if i >= 1024 {
                sum_sq += y * y;
            }
        }
        let rms = (sum_sq / (n - 1024) as f64).sqrt();
        println!("{freq:8.1} Hz  RMS = {rms:.4}");
    }
}
