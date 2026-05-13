//! Runtime processors.
//!
//! These are the things that actually run on samples. They own the
//! mutable state for a filter realisation; coefficients live separately
//! in [`crate::coeffs`] so they can be shared.

mod biquad;
mod delay;
mod direct_form;
mod fir;
mod gain;
mod sos;
mod state_space;

pub use biquad::{Biquad, BiquadState};
pub use delay::Delay;
pub use direct_form::DirectFormI;
pub use fir::{Fir, FirState};
pub use gain::Gain;
pub use sos::SosCascade;
pub use state_space::StateSpaceProcessor;

#[cfg(feature = "alloc")]
mod fir_dyn;
#[cfg(feature = "alloc")]
mod sos_dyn;

#[cfg(feature = "alloc")]
pub use fir_dyn::FirDyn;
#[cfg(feature = "alloc")]
pub use sos_dyn::SosDyn;
