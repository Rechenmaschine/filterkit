//! Tests for adaptive filters.

#![cfg(feature = "alloc")]

use filterkit::adaptive::{Lms, Nlms};
use filterkit::AdaptiveProcessor;

#[test]
fn lms_learns_a_simple_gain() {
    // Desired response: 2.5 * input. A 1-tap LMS should converge there.
    let mut lms = Lms::new(1, 0.05_f32);
    for n in 0..2000 {
        let x = ((n as f32) * 0.13).sin();
        let desired = 2.5 * x;
        let _ = lms.process_adapt(x, desired);
    }
    let learned = lms.w[0];
    assert!(
        (learned - 2.5).abs() < 1e-2,
        "LMS gain did not converge: got {learned}"
    );
}

#[test]
fn nlms_learns_under_changing_amplitude() {
    // Plant: y = 0.7*x[n] + 0.3*x[n-1]. Vary input level to stress NLMS.
    let mut nlms = Nlms::new(2, 0.5_f32, 1e-6);
    let mut prev = 0.0_f32;
    for n in 0..4000 {
        let amp = if n < 2000 { 0.1_f32 } else { 5.0_f32 };
        let x = amp * ((n as f32) * 0.27).sin();
        let desired = 0.7 * x + 0.3 * prev;
        prev = x;
        let _ = nlms.process_adapt(x, desired);
    }
    assert!((nlms.w[0] - 0.7).abs() < 0.05, "w0 = {}", nlms.w[0]);
    assert!((nlms.w[1] - 0.3).abs() < 0.05, "w1 = {}", nlms.w[1]);
}
