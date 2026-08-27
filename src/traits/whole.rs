use super::{Reset, SampleProcessor};

/// Processors that operate on a whole finite signal at once.
///
/// Unlike [`SampleProcessor`] or [`BlockProcessor`], a
/// `WholeSignalProcessor` can be non-causal, for example when implementing
/// zero-phase filtering or centered smoothing.
///
/// [`SampleProcessor`]: super::SampleProcessor
/// [`BlockProcessor`]: super::BlockProcessor
pub trait WholeSignalProcessor<I> {
    /// Sample produced for each input sample.
    type Output;

    /// Process the entire input into the entire output.
    ///
    /// # Panics
    ///
    /// Implementations should panic if `input.len() != output.len()`.
    fn process_whole(&mut self, input: &[I], output: &mut [Self::Output]);
}

/// Causal processors that can initialize themselves to the response of a
/// constant input.
///
/// After `reset_to_steady_input(x)`, the processor should behave as if it
/// had already processed an indefinitely long run of samples equal to
/// `x`. For example, an EMA should preload its previous output to `x`,
/// and an FIR should preload each delay slot to `x`.
///
/// This is different from [`Reset`], which clears state to the
/// processor's zero-input initial condition.
pub trait SteadyState<T>: SampleProcessor<T, Output = T> + Reset {
    /// Reset runtime state to the steady response for a constant input.
    fn reset_to_steady_input(&mut self, input: T);
}

/// Causal filters that provide the extra metadata needed by
/// forward/backward filtering.
///
/// `ForwardBackward` uses [`SteadyState::reset_to_steady_input`] for
/// SciPy-style endpoint initialization. It also needs a default edge
/// padding length, which depends on filter shape and order. This trait
/// keeps that filtfilt-specific policy separate from the more general
/// steady-state initialization capability.
pub trait FiltFiltKernel<T>: SteadyState<T> {
    /// Default edge-padding length for SciPy-style `filtfilt` padding.
    ///
    /// The value is counted in samples on each side of the signal. Built-in
    /// implementations follow SciPy's defaults: transfer-function filters
    /// use `3 * max(len(a), len(b))`, and SOS filters use SciPy's section
    /// count adjustment for poles/zeros at the origin.
    fn filtfilt_pad_len(&self) -> usize;
}
