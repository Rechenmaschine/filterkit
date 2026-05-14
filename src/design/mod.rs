//! Design helpers.
//!
//! Specifications that compile down to filter coefficients via the
//! [`crate::traits::Design`] trait.

mod auto;
mod biquad;
mod ema;
mod freq_response;
mod moving_average;
mod windowed_sinc;

pub use auto::{Lowpass, LowpassBuildError, LowpassSpec};
pub use biquad::{
    BiquadBandpassSpec, BiquadDesignError, BiquadHighpassSpec, BiquadLowpassSpec, BiquadNotchSpec,
};
pub use ema::{ExponentialAverageError, ExponentialAverageSpec};
pub use moving_average::{MovingAverageError, MovingAverageSpec};
pub use windowed_sinc::{Window, WindowedSincError, WindowedSincLowpassSpec};
