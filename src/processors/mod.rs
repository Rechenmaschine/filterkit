//! Stateful runtime processors.

mod biquad;
mod delay;
mod direct_form;
mod ema;
mod fir;
mod first_order;
mod gain;
mod sos;
mod state_space;

pub use biquad::Biquad;
pub use delay::Delay;
pub use direct_form::DirectFormI;
pub use ema::Ema;
pub use fir::Fir;
pub use first_order::FirstOrder;
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
