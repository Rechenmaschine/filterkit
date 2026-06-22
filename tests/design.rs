//! Tests for the design layer.

#![cfg(feature = "design")]

use approx::assert_relative_eq;
use filterkit::design::{
    BiquadBandpassSpec, BiquadHighpassSpec, BiquadLowpassSpec, BiquadNotchSpec,
    ExponentialAverageError, ExponentialAverageSpec, MovingAverageSpec, Window,
    WindowedSincLowpassSpec,
};
use filterkit::processors::{Biquad, Ema, Fir};
use filterkit::SampleProcessor;

#[test]
fn moving_average_is_uniform() {
    let c = MovingAverageSpec.design::<f32, 5>().unwrap();
    for t in c.b.iter() {
        assert_relative_eq!(*t, 0.2, epsilon = 1e-6);
    }
}

#[test]
fn windowed_sinc_lowpass_dc_gain_is_unity() {
    let spec = WindowedSincLowpassSpec {
        cutoff: 0.2,
        window: Window::Hamming,
    };
    let c = spec.design::<33>().unwrap();
    let dc: f64 = c.b.iter().sum();
    assert_relative_eq!(dc, 1.0, epsilon = 1e-9);
}

#[test]
fn windowed_sinc_lowpass_attenuates_above_cutoff() {
    let spec = WindowedSincLowpassSpec {
        cutoff: 0.1,
        window: Window::Blackman,
    };
    let c = spec.design::<63>().unwrap();
    let mut fir = Fir::new(c);
    // Feed a Nyquist-rate impulse pattern (+1, -1, +1, -1, ...) — should be
    // strongly attenuated by a 0.1-Nyquist lowpass.
    let mut max_out: f64 = 0.0;
    // skip initial transient
    for n in 0..1024 {
        let x = if n % 2 == 0 { 1.0 } else { -1.0 };
        let y = fir.process_sample(x);
        if n > 200 {
            max_out = max_out.max(y.abs());
        }
    }
    assert!(max_out < 1e-3, "high-freq leakage: {max_out}");
}

#[test]
fn biquad_lowpass_has_dc_gain_one() {
    let c = BiquadLowpassSpec { f0: 0.1, q: 0.707 }.design().unwrap();
    // H(1) = (b0 + b1 + b2) / (1 + a1 + a2)
    let num = c.b0 + c.b1 + c.b2;
    let den = 1.0 + c.a1 + c.a2;
    assert_relative_eq!(num / den, 1.0, epsilon = 1e-6);
}

#[test]
fn biquad_highpass_blocks_dc() {
    let c = BiquadHighpassSpec { f0: 0.1, q: 0.707 }.design().unwrap();
    let mut bq = Biquad::new(c);
    // Run DC for a long time; output should converge near zero.
    let mut last = 0.0_f64;
    for _ in 0..2000 {
        last = bq.process_sample(1.0);
    }
    assert!(last.abs() < 1e-3, "DC leakage in HP biquad: {last}");
}

#[test]
fn biquad_notch_kills_centre_frequency() {
    // A sinusoid at f0 should be attenuated.
    let f0 = 0.1;
    let c = BiquadNotchSpec { f0, q: 5.0 }.design().unwrap();
    let mut bq = Biquad::new(c);

    let mut peak: f64 = 0.0;
    let total = 4096;
    for n in 0..total {
        let t = n as f64;
        let x = (2.0 * core::f64::consts::PI * f0 * t).sin();
        let y = bq.process_sample(x);
        if n > 1024 {
            peak = peak.max(y.abs());
        }
    }
    assert!(peak < 0.1, "notch did not attenuate centre: {peak}");
}

#[test]
fn biquad_bandpass_design_succeeds() {
    let _ = BiquadBandpassSpec { f0: 0.2, q: 1.0 }.design().unwrap();
}

#[test]
fn ema_from_alpha_roundtrips() {
    let spec = ExponentialAverageSpec::from_alpha(0.25).unwrap();
    let alpha: f32 = spec.design().unwrap();
    assert_relative_eq!(alpha as f64, 0.25, epsilon = 1e-6);
}

#[test]
fn ema_from_alpha_rejects_out_of_range() {
    assert!(matches!(
        ExponentialAverageSpec::from_alpha(0.0),
        Err(ExponentialAverageError::InvalidAlpha)
    ));
    assert!(matches!(
        ExponentialAverageSpec::from_alpha(1.5),
        Err(ExponentialAverageError::InvalidAlpha)
    ));
    assert!(matches!(
        ExponentialAverageSpec::from_alpha(-0.1),
        Err(ExponentialAverageError::InvalidAlpha)
    ));
}

#[test]
fn ema_from_time_constant_matches_formula() {
    // tau = 0.01s @ 48 kHz: alpha = 1 - exp(-1/(0.01*48000)).
    let spec = ExponentialAverageSpec::from_time_constant(0.01, 48_000.0).unwrap();
    let expected = 1.0 - (-1.0_f64 / (0.01 * 48_000.0)).exp();
    assert_relative_eq!(spec.alpha(), expected, epsilon = 1e-12);
}

#[test]
fn ema_from_cutoff_hz_gives_3db_attenuation_at_cutoff() {
    // -3 dB → magnitude ≈ 1/sqrt(2). Run a long sine at fc and measure
    // steady-state amplitude.
    let fs = 48_000.0_f64;
    let fc = 1_000.0_f64;
    let spec = ExponentialAverageSpec::from_cutoff_hz(fc, fs).unwrap();
    let alpha: f64 = spec.design().unwrap();
    let mut p = Ema::new(alpha);

    let n_skip = 4096;
    let n_keep = 16384;
    for k in 0..n_skip {
        let x = (2.0 * core::f64::consts::PI * fc * k as f64 / fs).sin();
        let _ = p.process_sample(x);
    }
    let mut sum_sq = 0.0;
    for k in n_skip..(n_skip + n_keep) {
        let x = (2.0 * core::f64::consts::PI * fc * k as f64 / fs).sin();
        let y = p.process_sample(x);
        sum_sq += y * y;
    }
    let rms_out = (sum_sq / n_keep as f64).sqrt();
    let amp_out = rms_out * core::f64::consts::SQRT_2;
    // -3 dB = 0.7071...; allow ~5% tolerance for the impulse-invariant
    // approximation vs the true bilinear mapping.
    assert!(
        (amp_out - 1.0 / core::f64::consts::SQRT_2).abs() < 0.05,
        "magnitude at cutoff = {amp_out}, expected ~0.7071",
    );
}

#[test]
fn ema_rejects_cutoff_above_nyquist() {
    assert!(ExponentialAverageSpec::from_cutoff_hz(25_000.0, 48_000.0).is_err());
    assert!(ExponentialAverageSpec::from_cutoff_hz(-100.0, 48_000.0).is_err());
}

