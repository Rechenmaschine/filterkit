//! State estimators.
//!
//! Provides a linear Kalman filter behind the `kalman` feature.

mod kalman;

pub use kalman::{GaussianEstimate, KalmanFilter, KalmanModel};
