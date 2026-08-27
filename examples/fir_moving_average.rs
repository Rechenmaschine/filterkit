//! Apply a const-sized 5-tap moving average to a noisy ramp.
//!

use filterkit::design::MovingAverageSpec;
use filterkit::SampleProcessor;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

fn main() {
    let mut fir = MovingAverageSpec
        .build::<f32, 5>()
        .expect("moving average build");

    let mut rng = SmallRng::seed_from_u64(0xDEAD_BEEF);
    let xs: Vec<f32> = (0..40)
        .map(|i| {
            let noise = rng.random_range(-0.5_f32..0.5_f32);
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
