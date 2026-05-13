//! Smooth a noisy ramp with a one-pole EMA, configured three ways:
//! direct α, time constant, and cutoff frequency.
//!
//! Run with: `cargo run --example ema_smoothing`

use filterkit::design::ExponentialAverageSpec;
use filterkit::processors::OnePole;
use filterkit::SampleProcessor;

fn main() {
    let fs = 1_000.0_f64; // 1 kHz

    // Three equivalent ways to spec a one-pole at roughly the same
    // smoothing strength.
    let a_alpha = ExponentialAverageSpec::from_alpha(0.05).unwrap();
    let a_tau = ExponentialAverageSpec::from_time_constant(0.02, fs).unwrap();
    let a_fc = ExponentialAverageSpec::from_cutoff_hz(8.0, fs).unwrap();

    println!("alpha (direct)        = {:.4}", a_alpha.alpha());
    println!("alpha (tau = 20 ms)   = {:.4}", a_tau.alpha());
    println!("alpha (fc = 8 Hz)     = {:.4}", a_fc.alpha());

    let mut p_alpha = OnePole::new(a_alpha.design::<f64>().unwrap());
    let mut p_tau = OnePole::new(a_tau.design::<f64>().unwrap());
    let mut p_fc = OnePole::new(a_fc.design::<f64>().unwrap());

    // Noisy ramp.
    let mut rng_state: u32 = 0xC0FF_EE00;
    println!();
    println!("# n   noisy    α=0.05   τ=20ms    fc=8Hz");
    for n in 0..80 {
        rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = (rng_state >> 8) as f64 / (1 << 24) as f64 - 0.5;
        let x = n as f64 * 0.05 + noise * 0.6;
        let ya = p_alpha.process_sample(x);
        let yt = p_tau.process_sample(x);
        let yf = p_fc.process_sample(x);
        println!("{n:3}  {x:+.4}  {ya:+.4}  {yt:+.4}  {yf:+.4}");
    }
}
