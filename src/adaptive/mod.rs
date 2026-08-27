//! Adaptive filters implementing [`crate::traits::AdaptiveProcessor`].

#[cfg(feature = "alloc")]
mod lms;
#[cfg(feature = "alloc")]
mod nlms;

#[cfg(feature = "alloc")]
pub use lms::{Lms, LmsScalar};
#[cfg(feature = "alloc")]
pub use nlms::{Nlms, NlmsScalar};
