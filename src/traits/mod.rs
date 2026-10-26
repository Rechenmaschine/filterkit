//! Core processing traits.
//!
//! Traits define the interfaces used by processors.

mod ext;
mod sample;
mod whole;

pub use ext::ProcessorExt;
pub use sample::SampleProcessor;
pub use whole::{FiltFiltKernel, SteadyState, WholeSignalProcessor};

/// Clear runtime state of a processor *without* changing its coefficients
/// or parameters.
///
/// Most stateful processors implement this. After a [`Reset`], the
/// processor should behave as if it had just been constructed.
pub trait Reset {
    /// Clear runtime state.
    fn reset(&mut self);
}

/// External parameter or coefficient update.
///
/// Useful for retuning a filter at runtime without rebuilding it (e.g.
/// modulating biquad coefficients). The new coefficient block is moved in
/// and replaces the current set.
pub trait Retune<Coeffs> {
    /// Replace the active coefficient set.
    fn retune(&mut self, coeffs: Coeffs);
}
