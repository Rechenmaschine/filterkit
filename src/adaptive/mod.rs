//! Adaptive filters.
//!
//! Time-varying, signal-driven processors that adapt internal weights
//! against a desired reference. All implement
//! [`crate::traits::AdaptiveProcessor`].

#[cfg(feature = "alloc")]
mod lms;
#[cfg(feature = "alloc")]
mod nlms;

#[cfg(feature = "alloc")]
pub use lms::{Lms, LmsScalar};
#[cfg(feature = "alloc")]
pub use nlms::{Nlms, NlmsScalar};
