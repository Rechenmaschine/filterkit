//! Whole-signal processors.
//!
//! Algorithms that require the complete finite signal, such as zero-phase
//! filtering.

#[cfg(feature = "alloc")]
mod filtfilt;

#[cfg(feature = "alloc")]
pub use filtfilt::{ForwardBackward, PadType};
