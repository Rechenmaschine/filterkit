//! Apply a const-sized 5-tap moving average to a noisy ramp.
//!
//! Run with: `cargo run --example fir_moving_average`

use filterkit::design::MovingAverageSpec;
use filterkit::SampleProcessor;

fn main() {
    let mut fir = MovingAverageSpec
        .build::<f32, 5>()
        .expect("moving average build");

    let mut rng_state: u32 = 0xDEAD_BEEF;
    let xs: Vec<f32> = (0..40)
        .map(|i| {
            rng_state = rng_state.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = (rng_state >> 8) as f32 / (1 << 24) as f32 - 0.5;
            i as f32 * 0.1 + noise * 0.3
        })
        .collect();

    let mut ys = vec![0.0; xs.len()];
    fir.process_into(&xs, &mut ys);

    let mut fir2 = MovingAverageSpec
        .build::<f32, 5>()
        .expect("moving average build");
    let mut buffer = xs.clone();
    fir2.process_in_place(&mut buffer);

    println!("# n   noisy   process_into   process_in_place");
    for (i, ((x, y), z)) in xs.iter().zip(ys.iter()).zip(buffer.iter()).enumerate() {
        println!("{i:3}   {x:+.4}        {y:+.4}            {z:+.4}");
    }
}
