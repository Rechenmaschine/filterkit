//! Whole-signal processors.
//!
//! Algorithms that need the entire finite signal in hand: zero-phase
//! filtering (`filtfilt`), centered moving averages, batch spectral
//! processing. All implement [`crate::traits::WholeSignalProcessor`].

#[cfg(feature = "alloc")]
mod filtfilt;

#[cfg(feature = "alloc")]
pub use filtfilt::{ForwardBackward, PadType};
