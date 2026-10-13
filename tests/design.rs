//! Tests for the design layer.

#![cfg(feature = "design")]

use approx::assert_relative_eq;
use filterkit::design::{
    BiquadBandpassSpec, BiquadDesignError, BiquadHighpassSpec, BiquadLowpassSpec, BiquadNotchSpec,
    ExponentialAverageError, ExponentialAverageSpec, Window, WindowedSincLowpassSpec,
};
use filterkit::TransferFunction;

#[test]
fn windowed_sinc_lowpass() {
    let c = WindowedSincLowpassSpec {
        cutoff: 0.1,
        window: Window::Blackman,
    }
    .design::<63>()
    .unwrap();
    assert_relative_eq!(c.magnitude_at(0.0), 1.0, epsilon = 1e-9);
    assert!(c.magnitude_at(0.5) < 1e-3);
}

#[test]
fn biquad_designs() {
    let lp = BiquadLowpassSpec { f0: 0.1, q: 0.707 }.design().unwrap();
    assert_relative_eq!(lp.magnitude_at(0.0), 1.0, epsilon = 1e-9);

    let hp = BiquadHighpassSpec { f0: 0.1, q: 0.707 }.design().unwrap();
    assert!(hp.magnitude_at(0.0) < 1e-9);

    let notch = BiquadNotchSpec { f0: 0.1, q: 5.0 }.design().unwrap();
    assert!(notch.magnitude_at(0.1) < 1e-9);

    for q in [0.5, 1.0, 4.0] {
        let bp = BiquadBandpassSpec { f0: 0.2, q }.design().unwrap();
        assert_relative_eq!(bp.magnitude_at(0.2), 1.0, epsilon = 1e-9);
    }
}

#[test]
fn biquad_rejects_invalid_spec() {
    let spec = |f0, q| BiquadLowpassSpec { f0, q }.design();
    assert_eq!(spec(0.6, 1.0), Err(BiquadDesignError::InvalidFrequency));
    assert_eq!(spec(0.1, 0.0), Err(BiquadDesignError::InvalidQ));
    assert_eq!(spec(0.1, f64::NAN), Err(BiquadDesignError::InvalidQ));
}

#[test]
fn ema_from_cutoff_hz_is_near_3db_at_cutoff() {
    let alpha = ExponentialAverageSpec::from_cutoff_hz(1_000.0, 48_000.0)
        .unwrap()
        .alpha();
    let tf = TransferFunction::new([alpha], [alpha - 1.0]);
    assert_relative_eq!(
        tf.magnitude_at(1_000.0 / 48_000.0),
        core::f64::consts::FRAC_1_SQRT_2,
        epsilon = 0.01
    );
}

#[test]
fn ema_rejects_invalid_spec() {
    for alpha in [0.0, -0.1, 1.5, f64::NAN] {
        assert_eq!(
            ExponentialAverageSpec::from_alpha(alpha),
            Err(ExponentialAverageError::InvalidAlpha)
        );
    }
    assert!(ExponentialAverageSpec::from_cutoff_hz(25_000.0, 48_000.0).is_err());
    assert!(ExponentialAverageSpec::from_cutoff_hz(-100.0, 48_000.0).is_err());
}
