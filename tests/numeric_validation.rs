//! Numeric cross-validation: time-domain implementations match the
//! frequency-response math, and equivalent realisations of the same
//! transfer function agree.

#![cfg(feature = "design")]

use approx::assert_relative_eq;
use filterkit::coeffs::TransferFunction;
use filterkit::design::{BiquadLowpassSpec, MovingAverageSpec, Window, WindowedSincLowpassSpec};
use filterkit::processors::{Biquad, DirectFormI, Fir};
use filterkit::{Reset, SampleProcessor};

/// Drive a unit-amplitude sinusoid at normalised frequency `freq` and
/// return the steady-state output amplitude estimated via RMS. The RMS
/// of `A*sin(2π f n)` is `A/√2`, so output amplitude is `√2 * rms`. This
/// avoids the bias of `peak()` for high-frequency tones where the
/// integer-step sampling doesn't land on the analog peak.
fn measure_steady_state_amplitude<P>(f: &mut P, freq: f64) -> f64
where
    P: SampleProcessor<f64, Output = f64>,
{
    let n_skip = 2048;
    let n_keep = 8192;
    for k in 0..n_skip {
        let x = (2.0 * core::f64::consts::PI * freq * k as f64).sin();
        let _ = f.process_sample(x);
    }
    let mut sum_sq = 0.0;
    for k in n_skip..(n_skip + n_keep) {
        let x = (2.0 * core::f64::consts::PI * freq * k as f64).sin();
        let y = f.process_sample(x);
        sum_sq += y * y;
    }
    let rms = (sum_sq / n_keep as f64).sqrt();
    rms * core::f64::consts::SQRT_2
}

#[test]
fn biquad_lowpass_time_domain_matches_magnitude_at() {
    let spec = BiquadLowpassSpec { f0: 0.1, q: 0.707 };
    let c = spec.design().unwrap();
    let mut bq = Biquad::new(c);

    for &freq in &[0.001_f64, 0.05, 0.1, 0.2, 0.3] {
        bq.reset();
        let measured = measure_steady_state_amplitude(&mut bq, freq);
        let predicted = c.magnitude_at(freq);
        assert_relative_eq!(measured, predicted, max_relative = 0.01, epsilon = 1e-4);
    }
}

#[test]
fn fir_moving_average_magnitude_matches_closed_form() {
    let c = MovingAverageSpec.design::<f64, 5>().unwrap();
    // Closed-form: |H(f)| = |sin(π N f) / (N sin(π f))| for f != 0.
    let n = 5.0_f64;
    for &f in &[0.05_f64, 0.1, 0.2, 0.3] {
        let expected = ((core::f64::consts::PI * n * f).sin()
            / (n * (core::f64::consts::PI * f).sin()))
        .abs();
        let got = c.magnitude_at(f);
        assert_relative_eq!(got, expected, max_relative = 1e-10);
    }
}

#[test]
fn fir_impulse_response_matches_taps() {
    // Impulse response of an FIR is exactly its tap vector (delayed by
    // 0 samples on the first output).
    let c = WindowedSincLowpassSpec {
        cutoff: 0.1,
        window: Window::Hamming,
    }
    .design::<33>()
    .unwrap();
    let mut fir = Fir::new(c);
    let mut response = [0.0_f64; 33];
    response[0] = fir.process_sample(1.0);
    for i in 1..33 {
        response[i] = fir.process_sample(0.0);
    }
    for k in 0..33 {
        assert_relative_eq!(response[k], c.b[k], epsilon = 1e-12);
    }
}

#[test]
fn directform1_equals_biquad_for_same_transfer_function() {
    let bq_c = BiquadLowpassSpec { f0: 0.15, q: 0.707 }.design().unwrap();

    // Same transfer function in TF form: H = (b0 + b1 z^-1 + b2 z^-2)
    //                                       / (1 + a1 z^-1 + a2 z^-2)
    let tf: TransferFunction<f64, 3, 2> = bq_c.into();
    let mut bq = Biquad::new(bq_c);
    let mut df1 = DirectFormI::new(tf);

    // Compare on a deterministic random-ish input.
    let mut x = 0.0_f64;
    for n in 0..1024 {
        x = (n as f64 * 0.137).sin() * 0.5 + (n as f64 * 0.41).cos() * 0.3;
        let y_bq = bq.process_sample(x);
        let y_df = df1.process_sample(x);
        assert_relative_eq!(y_bq, y_df, max_relative = 1e-10, epsilon = 1e-12);
    }
    let _ = x;
}

#[test]
fn fir_with_history_matches_continuation_of_block() {
    // Run an FIR over a long signal, snapshot at midpoint, then build a
    // second FIR with the appropriate history and continue. Outputs
    // must match exactly.
    let c = WindowedSincLowpassSpec {
        cutoff: 0.2,
        window: Window::Hann,
    }
    .design::<16>()
    .unwrap();

    let signal: Vec<f64> = (0..200)
        .map(|n| (n as f64 * 0.31).sin() + (n as f64 * 0.77).cos())
        .collect();
    let split = 100_usize;

    let mut full = Fir::new(c);
    let full_out: Vec<f64> = signal.iter().map(|&x| full.process_sample(x)).collect();

    // First half through fresh filter.
    let mut a = Fir::new(c);
    for &x in &signal[..split] {
        let _ = a.process_sample(x);
    }

    // Build history snapshot from signal[..split].
    let mut history = [0.0_f64; 16];
    for k in 0..16 {
        let idx = split as isize - 1 - k as isize;
        history[k] = if idx >= 0 { signal[idx as usize] } else { 0.0 };
    }
    let mut b = Fir::with_history(c, history);
    for n in split..signal.len() {
        let y = b.process_sample(signal[n]);
        assert_relative_eq!(y, full_out[n], max_relative = 1e-10, epsilon = 1e-12);
    }
}
