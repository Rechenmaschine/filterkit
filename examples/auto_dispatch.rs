//! Build the same logical lowpass at order 1 and order 2; let the
//! library pick the kernel for each. Same user-facing code path, two
//! different concrete implementations under the hood.
//!
//! Run with: `cargo run --example auto_dispatch`

use filterkit::design::{Lowpass, LowpassSpec};
use filterkit::processors::{Biquad, OnePole};
use filterkit::SampleProcessor;

fn main() {
    let fs = 48_000.0_f64;
    let fc = 200.0_f64;

    // ---- AUTO-DISPATCH PATH ----
    // Same call shape, different kernels selected by `order`.
    let mut lp1: Lowpass<f64> = LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 1 }
        .build()
        .unwrap();
    let mut lp2: Lowpass<f64> = LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 2 }
        .build()
        .unwrap();

    println!("order=1 kernel: {}", kernel_name(&lp1));
    println!("order=2 kernel: {}", kernel_name(&lp2));

    // ---- EXPLICIT KERNEL PATH ----
    // Same spec, but force a specific kernel type. No enum dispatch;
    // these are concrete OnePole<f64> and Biquad<f64> values you can
    // hand directly to combinators or store in your own structs.
    let _forced_one_pole: OnePole<f64> =
        LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 99 /* ignored */ }
            .as_one_pole()
            .unwrap();
    let _forced_biquad: Biquad<f64> =
        LowpassSpec { cutoff_hz: fc, sample_rate: fs, order: 99 /* ignored */ }
            .as_biquad()
            .unwrap();
    println!("explicit OnePole and Biquad materialised from the same LowpassSpec");
    println!();

    // Sweep some frequencies, measure steady-state RMS through each.
    for &f in &[20.0_f64, 100.0, 200.0, 1_000.0, 5_000.0] {
        let rms1 = steady_state_rms(&mut lp1, f, fs);
        let rms2 = steady_state_rms(&mut lp2, f, fs);
        println!(
            "f = {f:>7.1} Hz   order1 RMS = {rms1:.4}   order2 RMS = {rms2:.4}",
        );
    }
}

fn kernel_name<T>(lp: &Lowpass<T>) -> &'static str {
    match lp {
        Lowpass::OnePole(_) => "OnePole",
        Lowpass::Biquad(_) => "Biquad",
        // Lowpass is #[non_exhaustive] — future variants land here.
        _ => "other",
    }
}

fn steady_state_rms(lp: &mut Lowpass<f64>, freq: f64, fs: f64) -> f64 {
    let n_skip = 2048;
    let n_keep = 8192;
    for k in 0..n_skip {
        let x = (2.0 * core::f64::consts::PI * freq * k as f64 / fs).sin();
        let _ = lp.process_sample(x);
    }
    let mut sum_sq = 0.0;
    for k in n_skip..(n_skip + n_keep) {
        let x = (2.0 * core::f64::consts::PI * freq * k as f64 / fs).sin();
        let y = lp.process_sample(x);
        sum_sq += y * y;
    }
    (sum_sq / n_keep as f64).sqrt()
}
