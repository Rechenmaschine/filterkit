//! Kalman filter smoke tests. Compiled only with the `kalman` feature;
//! run with `cargo test --features kalman`.
#![cfg(feature = "kalman")]

use filterkit::{KalmanFilter, KalmanModel, SampleProcessor};
use nalgebra::{Matrix1, Matrix2, RowVector2, SVector, Vector2};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

/// A 1-D constant-velocity tracker should estimate position with lower
/// error than the raw noisy measurements it is fed.
#[test]
fn constant_velocity_beats_raw_measurements() {
    let dt = 1.0;
    // State = [position, velocity]; we measure position only.
    let f = Matrix2::new(1.0, dt, 0.0, 1.0);
    let h = RowVector2::new(1.0, 0.0); // 1×2
    let q = Matrix2::new(1e-4, 0.0, 0.0, 1e-4);
    let r = Matrix1::new(0.25); // measurement variance

    let model = KalmanModel::new(f, h, q, r);
    let prior_cov = Matrix2::identity() * 1.0;
    let mut kf = KalmanFilter::new(model, Vector2::new(0.0, 0.0), prior_cov);

    let true_velocity = 0.5;
    let meas_amp = 1.0; // ±0.5 noise band
    let mut rng = SmallRng::seed_from_u64(0xC0FF_EE00);

    let mut sse_filtered = 0.0;
    let mut sse_raw = 0.0;
    let steps = 200;

    for k in 0..steps {
        let true_pos = true_velocity * k as f64;
        let noise = rng.random_range(-0.5_f64..0.5_f64) * meas_amp;
        let z = true_pos + noise;

        let est = kf.process_sample(SVector::<f64, 1>::new(z));
        let filtered_pos = est.mean[0];

        // Skip the warm-up while the filter locks on.
        if k >= 20 {
            sse_filtered += (filtered_pos - true_pos).powi(2);
            sse_raw += (z - true_pos).powi(2);
        }
    }

    let rmse_filtered = (sse_filtered / (steps - 20) as f64).sqrt();
    let rmse_raw = (sse_raw / (steps - 20) as f64).sqrt();

    assert!(
        rmse_filtered < rmse_raw,
        "filter should reduce error: filtered={rmse_filtered:.4} raw={rmse_raw:.4}",
    );
    // Posterior covariance must stay finite and positive on the diagonal.
    let p = kf.estimate().cov;
    assert!(p[(0, 0)] > 0.0 && p[(0, 0)].is_finite());
}

/// A bare `predict` with no measurement must propagate the mean through
/// `F` and grow the covariance by `Q`.
#[test]
fn predict_without_measurement_propagates_state() {
    let f = Matrix2::new(1.0, 1.0, 0.0, 1.0);
    let model = KalmanModel::new(
        f,
        RowVector2::new(1.0, 0.0),
        Matrix2::identity() * 1e-3,
        Matrix1::new(0.1),
    );
    let mut kf = KalmanFilter::new(model, Vector2::new(2.0_f64, 3.0), Matrix2::identity());

    kf.predict();
    let est = kf.estimate();
    // x = F x = [2 + 3, 3] = [5, 3]
    assert!((est.mean[0] - 5.0).abs() < 1e-12);
    assert!((est.mean[1] - 3.0).abs() < 1e-12);
    // Covariance should have grown.
    assert!(est.cov[(0, 0)] > 1.0);
}
