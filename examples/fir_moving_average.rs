//! Apply a const-sized 5-tap moving average to a noisy ramp.
//!
//! Run with: `cargo run --example fir_moving_average`

use filterkit::design::MovingAverageSpec;
use filterkit::processors::Fir;
use filterkit::SampleProcessor;

fn main() {
    // 5-tap MA via the design layer.
    let coeffs = MovingAverageSpec
        .design::<f32, 5>()
        .expect("moving average design");
    let mut fir = Fir::new(coeffs);

    // Noisy ramp signal.
    let mut rng_state: u32 = 0xDEAD_BEEF;
    let xs: Vec<f32> = (0..40)
        .map(|i| {
            // simple LCG noise in [-0.5, 0.5]
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = (rng_state >> 8) as f32 / (1 << 24) as f32 - 0.5;
            i as f32 * 0.1 + noise * 0.3
        })
        .collect();

    let mut ys = vec![0.0; xs.len()];
    fir.process_into(&xs, &mut ys);

    println!("# n  noisy  smoothed");
    for (i, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
        println!("{i:3}  {x:+.4}  {y:+.4}");
    }
}
