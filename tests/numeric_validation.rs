//! Cross-checks: processors match their frequency response, and equivalent
//! realisations agree.

#![cfg(feature = "design")]

use approx::assert_relative_eq;
use filterkit::coeffs::TransferFunction;
use filterkit::design::{BiquadLowpassSpec, MovingAverageSpec, Window, WindowedSincLowpassSpec};
use filterkit::processors::{Biquad, DirectFormI, Fir};
use filterkit::{Reset, SampleProcessor};

/// Steady-state output amplitude for a unit sine at `freq`, via RMS.
fn steady_state_amplitude<P>(p: &mut P, freq: f64) -> f64
where
    P: SampleProcessor<f64, Output = f64>,
{
    let (skip, keep) = (2048, 8192);
    let x = |k: usize| (2.0 * core::f64::consts::PI * freq * k as f64).sin();
    for k in 0..skip {
        p.process_sample(x(k));
    }
    let sum_sq: f64 = (skip..skip + keep)
        .map(|k| p.process_sample(x(k)).powi(2))
        .sum();
    (sum_sq / keep as f64).sqrt() * core::f64::consts::SQRT_2
}

#[test]
fn biquad_time_domain_matches_magnitude_at() {
    let c = BiquadLowpassSpec { f0: 0.1, q: 0.707 }.design().unwrap();
    let mut bq = Biquad::new(c);
    for freq in [0.001, 0.05, 0.1, 0.2, 0.3] {
        bq.reset();
        let measured = steady_state_amplitude(&mut bq, freq);
        assert_relative_eq!(
            measured,
            c.magnitude_at(freq),
            max_relative = 0.01,
            epsilon = 1e-4
        );
    }
}

#[test]
fn moving_average_magnitude_matches_closed_form() {
    let c = MovingAverageSpec.design::<f64, 5>().unwrap();
    let pi = core::f64::consts::PI;
    for f in [0.05, 0.1, 0.2, 0.3] {
        let expected = ((pi * 5.0 * f).sin() / (5.0 * (pi * f).sin())).abs();
        assert_relative_eq!(c.magnitude_at(f), expected, max_relative = 1e-10);
    }
}

#[test]
fn direct_form_i_equals_biquad() {
    let c = BiquadLowpassSpec { f0: 0.15, q: 0.707 }.design().unwrap();
    let tf: TransferFunction<f64, 3, 2> = c.into();
    let mut bq = Biquad::new(c);
    let mut df1 = DirectFormI::new(tf);
    for n in 0..1024 {
        let x = (n as f64 * 0.137).sin() * 0.5 + (n as f64 * 0.41).cos() * 0.3;
        assert_relative_eq!(
            bq.process_sample(x),
            df1.process_sample(x),
            max_relative = 1e-10,
            epsilon = 1e-12
        );
    }
}

#[test]
fn fir_with_history_continues_block() {
    let c = WindowedSincLowpassSpec {
        cutoff: 0.2,
        window: Window::Hann,
    }
    .design::<16>()
    .unwrap();
    let signal: Vec<f64> = (0..200)
        .map(|n| (n as f64 * 0.31).sin() + (n as f64 * 0.77).cos())
        .collect();
    let split = 100;

    let mut full = Fir::new(c);
    let full_out: Vec<f64> = signal.iter().map(|&x| full.process_sample(x)).collect();

    // history[0] is the most recent sample.
    let history: [f64; 16] = core::array::from_fn(|k| signal[split - 1 - k]);
    let mut resumed = Fir::with_history(c, history);
    for n in split..signal.len() {
        assert_relative_eq!(
            resumed.process_sample(signal[n]),
            full_out[n],
            max_relative = 1e-10,
            epsilon = 1e-12
        );
    }
}
