//! Tests for whole-signal processors.

#![cfg(feature = "alloc")]

use approx::assert_relative_eq;
use filterkit::processors::Biquad;
use filterkit::traits::WholeSignalProcessor;
use filterkit::whole::ForwardBackward;
use filterkit::BiquadCoeffs;

#[test]
fn forward_backward_with_identity_is_identity() {
    let bq = Biquad::new(BiquadCoeffs::identity());
    let mut filt = ForwardBackward::new(bq);
    let xs: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mut ys = [0.0_f32; 5];
    filt.process_whole(&xs, &mut ys);
    for i in 0..5 {
        assert_relative_eq!(ys[i], xs[i], epsilon = 1e-6);
    }
}

#[test]
#[cfg(feature = "design")]
fn forward_backward_zero_phase_lowpass() {
    use filterkit::design::BiquadLowpassSpec;

    let c = BiquadLowpassSpec { f0: 0.05, q: 0.707 }.design().unwrap();
    let bq = Biquad::new(c);
    let mut filt = ForwardBackward::new(bq);

    // Synthesize a sinusoid well below cutoff. Phase should be preserved.
    let n = 256;
    let f = 0.01;
    let xs: Vec<f64> = (0..n)
        .map(|i| (2.0 * core::f64::consts::PI * f * i as f64).sin())
        .collect();
    let mut ys = vec![0.0; n];
    filt.process_whole(&xs, &mut ys);

    // Compare mid-signal samples (away from boundary transients).
    let mid = n / 2;
    let want = xs[mid];
    let got = ys[mid];
    assert!(
        (got - want).abs() < 0.2,
        "zero-phase output diverged: want {want}, got {got}"
    );
}
