//! Variable-rate stream processors.
//!
//! Includes decimators, interpolators, and polyphase resamplers.

mod decimator;
mod interpolator;
mod resampler;

pub use decimator::Decimator;
pub use interpolator::Interpolator;

#[cfg(feature = "alloc")]
pub use resampler::PolyphaseResampler;
