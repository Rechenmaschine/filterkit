//! Variable-rate stream processors.
//!
//! Decimators, interpolators, and polyphase resamplers — anything where
//! input and output rates differ. All implement
//! [`crate::traits::StreamProcessor`].

mod decimator;
mod interpolator;
mod resampler;

pub use decimator::Decimator;
pub use interpolator::Interpolator;

#[cfg(feature = "alloc")]
pub use resampler::PolyphaseResampler;
