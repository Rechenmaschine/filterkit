//! State estimators.
//!
//! Unlike the LTI [`crate::processors`], these track a *belief* about a
//! hidden state — a mean and its covariance — rather than a single
//! filtered value. This module provides a linear Kalman filter.
//!
//! Requires the `kalman` feature (pulls in `nalgebra` for matrix
//! algebra).

mod kalman;

pub use kalman::{GaussianEstimate, KalmanFilter, KalmanModel};
