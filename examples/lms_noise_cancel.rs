//! Toy adaptive noise cancellation with [`Lms`].
//!
//! Plant: corrupt a sinusoid with a filtered noise source; let LMS adapt
//! to the impulse response of the noise path, subtract its prediction
//! from the corrupted observation, and watch the residual converge to
//! the clean signal.
//!
//! [`Lms`]: filterkit::adaptive::Lms

use filterkit::adaptive::Lms;
use filterkit::AdaptiveProcessor;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

fn main() {
    // Plant impulse response we want LMS to learn.
    let plant = [0.6_f32, 0.3, 0.1];

    let mut lms = Lms::new(plant.len(), 0.02_f32);

    let mut prev_a = 0.0;
    let mut prev_b = 0.0;

    // Drive with pseudo-random reference noise; measure plant output as
    // the "interference" we want to cancel from observation. Clean
    // signal is the 1 kHz sinusoid at 48 kHz.
    let fs = 48_000.0_f32;
    let f = 1_000.0;

    let mut rng = SmallRng::seed_from_u64(0xABAD_CAFE);
    let mut residual_after_settle = 0.0_f32;
    let mut count = 0_usize;

    for n in 0..20_000 {
        let noise = rng.random_range(-0.5_f32..0.5_f32);

        let clean = (2.0 * core::f32::consts::PI * f * n as f32 / fs).sin();
        // Filtered noise = plant * noise (length-3 FIR by hand)
        let interference = plant[0] * noise + plant[1] * prev_a + plant[2] * prev_b;
        prev_b = prev_a;
        prev_a = noise;
        let observation = clean + interference;

        let predicted = lms.process_sample(noise);
        let residual = lms.adapt(observation, predicted); // err = observation - predicted

        if n > 15_000 {
            // After ~15k samples the LMS weights should be near `plant`.
            let cancellation_error = residual - clean;
            residual_after_settle += cancellation_error * cancellation_error;
            count += 1;
        }
    }

    let mse = residual_after_settle / count.max(1) as f32;
    println!("Steady-state MSE between residual and clean signal: {mse:.6}");
    print!("Learned weights:");
    for w in lms.w.iter() {
        print!(" {w:+.4}");
    }
    println!();
    print!("Target plant:   ");
    for w in plant.iter() {
        print!(" {w:+.4}");
    }
    println!();
}
