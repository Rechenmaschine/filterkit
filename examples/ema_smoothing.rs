//! Smooth a noisy ramp with an EMA, configured three ways:
//! direct α, time constant, and cutoff frequency.
//!

use filterkit::design::ExponentialAverageSpec;
use filterkit::SampleProcessor;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

fn main() {
    let fs = 1_000.0_f64; // 1 kHz

    let a_alpha = ExponentialAverageSpec::from_alpha(0.05).unwrap();
    let a_tau = ExponentialAverageSpec::from_time_constant(0.02, fs).unwrap();
    let a_fc = ExponentialAverageSpec::from_cutoff_hz(8.0, fs).unwrap();

    println!("alpha (direct)        = {:.4}", a_alpha.alpha());
    println!("alpha (tau = 20 ms)   = {:.4}", a_tau.alpha());
    println!("alpha (fc = 8 Hz)     = {:.4}", a_fc.alpha());

    let mut p_alpha = a_alpha.build::<f64>().unwrap();
    let mut p_tau = a_tau.build::<f64>().unwrap();
    let mut p_fc = a_fc.build::<f64>().unwrap();

    let mut rng = SmallRng::seed_from_u64(0xC0FF_EE00);
    println!();
    println!("# n   noisy    α=0.05   τ=20ms    fc=8Hz");
    for n in 0..80 {
        let noise = rng.random_range(-0.5_f64..0.5_f64);
        let x = n as f64 * 0.05 + noise * 0.6;
        let ya = p_alpha.process_sample(x);
        let yt = p_tau.process_sample(x);
        let yf = p_fc.process_sample(x);
        println!("{n:3}  {x:+.4}  {ya:+.4}  {yt:+.4}  {yf:+.4}");
    }
}
