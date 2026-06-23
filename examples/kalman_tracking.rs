//! Track a 1-D constant-velocity target from noisy position measurements
//! with a linear Kalman filter, then compare the filtered estimate to the
//! raw measurements.
//!
//! Run with: `cargo run --example kalman_tracking --features kalman`

use filterkit::{KalmanFilter, KalmanModel, SampleProcessor};
use nalgebra::{Matrix1, Matrix2, RowVector2, SVector};

fn main() {
    let dt = 1.0_f64;

    // State = [position, velocity]. We observe position only.
    let f = Matrix2::new(1.0, dt, 0.0, 1.0); // x' = F x
    let h = RowVector2::new(1.0, 0.0); // z = H x
    let q = Matrix2::new(1e-4, 0.0, 0.0, 1e-4); // process noise
    let r = Matrix1::new(0.25); // measurement variance (σ = 0.5)

    let model = KalmanModel::new(f, h, q, r);
    // Start uncertain: zero mean, broad covariance.
    let mut kf = KalmanFilter::with_prior_cov(model, Matrix2::identity() * 10.0);

    let true_velocity = 0.5;
    let mut rng: u32 = 0xC0FF_EE00;

    println!("# k   true     meas    filt_pos  filt_vel");
    let mut sse_filt = 0.0;
    let mut sse_raw = 0.0;
    let warmup = 10;
    let steps = 40;

    for k in 0..steps {
        let true_pos = true_velocity * k as f64;

        // Deterministic ±0.5 measurement noise.
        rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let noise = (rng >> 8) as f64 / (1u32 << 24) as f64 - 0.5;
        let z = true_pos + noise;

        let est = kf.process_sample(SVector::<f64, 1>::new(z));
        let (filt_pos, filt_vel) = (est.mean[0], est.mean[1]);

        println!("{k:3}  {true_pos:+.4}  {z:+.4}  {filt_pos:+.4}   {filt_vel:+.4}");

        if k >= warmup {
            sse_filt += (filt_pos - true_pos).powi(2);
            sse_raw += (z - true_pos).powi(2);
        }
    }

    let n = (steps - warmup) as f64;
    println!();
    println!("post-warmup RMSE  raw = {:.4}", (sse_raw / n).sqrt());
    println!("post-warmup RMSE  filtered = {:.4}", (sse_filt / n).sqrt());
    println!("recovered velocity ≈ {:.4} (true {true_velocity})", {
        kf.estimate().mean[1]
    });
}
