//! State estimators.
//!
//! Unlike the LTI [`crate::processors`], these track a *belief* about a
//! hidden state — a mean and its covariance — rather than a single
//! filtered value. The first family is the linear Kalman filter; the
//! same predict/update shape is intended to host extended/unscented
//! variants and the RTS smoother as they land.
//!
//! Requires the `kalman` feature (pulls in `nalgebra` for matrix
//! algebra).

mod kalman;

pub use kalman::{GaussianEstimate, KalmanFilter, KalmanModel};
