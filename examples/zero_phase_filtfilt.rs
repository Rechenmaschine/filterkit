//! Zero-phase filtering with [`ForwardBackward`].
//!
//! Compare the phase of the causal output and the forward/backward
//! output for a low-frequency tone.
//!
//! [`ForwardBackward`]: filterkit::whole::ForwardBackward

use filterkit::design::BiquadLowpassSpec;
use filterkit::processors::Biquad;
use filterkit::traits::WholeSignalProcessor;
use filterkit::whole::ForwardBackward;
use filterkit::SampleProcessor;

fn main() {
    let fs = 1_000.0_f64;
    let f = 50.0;
    let n = 512_usize;
    let xs: Vec<f64> = (0..n)
        .map(|i| (2.0 * core::f64::consts::PI * f * i as f64 / fs).sin())
        .collect();

    // Build the same biquad lowpass at 80 Hz cutoff twice.
    let mut causal = Biquad::new(
        BiquadLowpassSpec {
            f0: 80.0 / fs,
            q: 0.707,
        }
        .design()
        .unwrap(),
    );
    let mut zero_phase = ForwardBackward::new(Biquad::new(
        BiquadLowpassSpec {
            f0: 80.0 / fs,
            q: 0.707,
        }
        .design()
        .unwrap(),
    ));

    let mut causal_y = vec![0.0; n];
    for (i, x) in xs.iter().enumerate() {
        causal_y[i] = causal.process_sample(*x);
    }

    let mut zp_y = vec![0.0; n];
    zero_phase.process_whole(&xs, &mut zp_y);

    println!("# n  input  causal  zero_phase");
    // Skip the first 20 samples to let causal settle.
    for i in 20..40 {
        println!("{i:3}  {:+.4}  {:+.4}  {:+.4}", xs[i], causal_y[i], zp_y[i]);
    }
}
