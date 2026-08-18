//! Core execution traits.
//!
//! The traits in this module describe *shapes* of processing, not specific
//! filter families. Concrete processors implement whichever traits match
//! their natural API — most causal LTI filters implement
//! [`SampleProcessor`], block-native algorithms implement
//! [`BlockProcessor`], variable-rate operators implement
//! [`StreamProcessor`], and whole-array algorithms implement
//! [`WholeSignalProcessor`].
//!
//! Numeric bounds (e.g. `num_traits::Float`) are deliberately *not* placed
//! on these traits; they live on concrete implementations so that
//! non-float processors (integer DSP, fixed-point, symbol streams) remain
//! expressible.

mod block;
mod ext;
mod prepare;
mod sample;
mod stream;
mod whole;

pub use block::BlockProcessor;
pub use ext::ProcessorExt;
pub use prepare::{Prepare, ProcessSpec};
pub use sample::SampleProcessor;
pub use stream::{StreamProcessor, StreamStatus};
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

/// A high-level specification that can be turned into something concrete
/// — typically a coefficient block, a model, or a ready-to-run processor.
///
/// This is the bridge between the *design* layer (filter specs in
/// [`crate::design`]) and the *runtime* layer.
pub trait Design {
    /// What `design` produces (e.g. [`crate::coeffs::FirCoeffs`]).
    type Output;
    /// Error type for design failures (e.g. infeasible spec).
    type Error;

    /// Run the design and produce the output.
    fn design(&self) -> Result<Self::Output, Self::Error>;
}

/// A signal-driven, time-varying processor (e.g. LMS, NLMS, RLS).
///
/// Adaptive processors differ from LTI [`SampleProcessor`]s in that
/// `adapt` is called with a *desired* sample to update internal weights.
pub trait AdaptiveProcessor<I>: Reset {
    /// The sample type produced by the filter.
    type Output;

    /// Run the current weights against a sample without adapting.
    fn process_sample(&mut self, input: I) -> Self::Output;

    /// Adapt internal weights given the just-produced `output` and the
    /// `desired` reference. Returns the error signal used to drive the
    /// update.
    fn adapt(&mut self, desired: Self::Output, output: Self::Output) -> Self::Output;

    /// Convenience: process one sample and adapt against a desired
    /// reference, returning `(output, error)`.
    ///
    /// Adaptive processors commonly require [`process_sample`] to be
    /// called immediately before [`adapt`] (the latter reads internal
    /// state populated by the former). This helper enforces that
    /// ordering.
    ///
    /// `Self::Output: Copy` is required so the same `output` value can
    /// be returned and passed to `adapt`.
    ///
    /// [`process_sample`]: AdaptiveProcessor::process_sample
    /// [`adapt`]: AdaptiveProcessor::adapt
    fn process_adapt(&mut self, input: I, desired: Self::Output) -> (Self::Output, Self::Output)
    where
        I: Copy,
        Self::Output: Copy,
    {
        let output = self.process_sample(input);
        let error = self.adapt(desired, output);
        (output, error)
    }
}
