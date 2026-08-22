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

#[test]
#[cfg(feature = "design")]
#[allow(clippy::excessive_precision)]
fn forward_backward_matches_scipy_pad_method_for_biquad() {
    use filterkit::design::BiquadLowpassSpec;

    let c = BiquadLowpassSpec { f0: 0.05, q: 0.707 }.design().unwrap();
    let mut filt = ForwardBackward::new(Biquad::new(c)).with_pad_len(9);

    let xs = [
        0.2_f64, -0.1, 0.4, 0.8, 0.3, -0.5, -0.2, 0.0, 0.6, 1.0, 0.7, 0.1, -0.4, -0.3, 0.2, 0.5,
    ];
    let scipy = [
        0.21883059454556961,
        0.2217421605305924,
        0.22099171400874079,
        0.2171552338944355,
        0.2121850673548584,
        0.21025640321330244,
        0.2154067005618393,
        0.22839051385540454,
        0.24630946640300974,
        0.2652631799213554,
        0.28396176935620143,
        0.30555402225701428,
        0.33585133720512927,
        0.37911947998753048,
        0.43473378233375448,
        0.49693553451328193,
    ];

    let mut ys = [0.0_f64; 16];
    filt.process_whole(&xs, &mut ys);

    for i in 0..xs.len() {
        assert_relative_eq!(ys[i], scipy[i], epsilon = 1e-12);
    }
}
