//! Coefficient and parameter types for filter representations.
//!
//! Coefficients are separate from processor state and can be shared.

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
