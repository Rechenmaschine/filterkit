//! Filter representations — the *what*, not the *how*.
//!
//! These types describe an LTI system mathematically. They do not run
//! anything. To execute, pair them with a processor from
//! [`crate::processors`]:
//!
//! - [`FirCoeffs`] + [`crate::processors::Fir`]
//! - [`BiquadCoeffs`] + [`crate::processors::Biquad`]
//! - [`SosCoeffs`] + [`crate::processors::SosCascade`]
//! - [`TransferFunction`] + [`crate::processors::DirectFormI`]
//! - [`StateSpace`] + [`crate::processors::StateSpaceProcessor`]
//!
//! Coefficient values are kept separate from runtime state so they may
//! be shared (`&'static`, multiple voices, etc.) while state is not.

mod biquad;
mod fir;
mod sos;
mod state_space;
mod tf;

pub use biquad::BiquadCoeffs;
pub use fir::FirCoeffs;
pub use sos::SosCoeffs;
pub use state_space::StateSpace;
pub use tf::TransferFunction;
