//! Design helpers.
//!
//! Specifications that compile down to filter coefficients or processors.

mod biquad;
mod ema;
mod freq_response;
mod moving_average;
mod windowed_sinc;

pub use biquad::{
    BiquadBandpassSpec, BiquadDesignError, BiquadHighpassSpec, BiquadLowpassSpec, BiquadNotchSpec,
    BiquadScalar,
};
pub use ema::{ExponentialAverageError, ExponentialAverageSpec};
pub use moving_average::{MovingAverageError, MovingAverageSpec};
pub use windowed_sinc::{Window, WindowedSincError, WindowedSincLowpassSpec};
