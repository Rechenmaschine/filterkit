use super::Reset;

/// A causal, same-rate processor that consumes one input sample and
/// produces one output sample.
///
/// Most fixed LTI filters and many DSP primitives fit this shape: FIR,
/// IIR (direct forms), biquads, SOS cascades, gains, delays, integrators,
/// DC blockers, one-pole filters, and so on. The associated [`Output`]
/// type lets [`SampleProcessor`]s compose without forcing input and
/// output to match.
///
/// # Batch methods
///
/// Two default methods provide batch processing on top of
/// [`process_sample`]. Both are overridable — concrete kernels that
/// can do block-rate work (e.g. enum dispatch hoisted out of the inner
/// loop, SIMD, FFT) should override these directly.
///
/// - [`process_into`] takes separate input/output slices.
/// - [`process_in_place`] mutates a single buffer; available only when
///   `Self::Output = I` via a method-level constraint.
///
/// [`Output`]: SampleProcessor::Output
/// [`process_sample`]: SampleProcessor::process_sample
/// [`process_into`]: SampleProcessor::process_into
/// [`process_in_place`]: SampleProcessor::process_in_place
pub trait SampleProcessor<I>: Reset {
    /// Sample produced for each input sample.
    type Output;

    /// Process exactly one input sample.
    fn process_sample(&mut self, input: I) -> Self::Output;

    /// Convenience batch wrapper: process one slice into another of the
    /// same length. Default implementation calls [`process_sample`] in a
    /// tight loop; specialised processors are free to override.
    ///
    /// # Panics
    ///
    /// Panics if `input.len() != output.len()`.
    ///
    /// [`process_sample`]: SampleProcessor::process_sample
    fn process_into(&mut self, input: &[I], output: &mut [Self::Output])
    where
        I: Copy,
    {
        assert_eq!(
            input.len(),
            output.len(),
            "process_into: input and output must have equal length",
        );

        for (x, y) in input.iter().copied().zip(output.iter_mut()) {
            *y = self.process_sample(x);
        }
    }

    /// Process samples in-place. Available only when `Self::Output = I`
    /// (read-modify-write requires the produced sample to fit back in
    /// the slot it came from). Default implementation calls
    /// [`process_sample`] over each element; concrete kernels are free
    /// to override with a faster loop.
    fn process_in_place(&mut self, buffer: &mut [I])
    where
        I: Copy,
        Self: SampleProcessor<I, Output = I>,
    {
        for x in buffer.iter_mut() {
            *x = self.process_sample(*x);
        }
    }
}

/// Backwards-compatible alias: any `SampleProcessor<T, Output = T>`
/// supports [`SampleProcessor::process_in_place`] directly. Kept as a
/// named trait so users can write `where F: SampleFilter<T>` as a
/// readable bound instead of the longer associated-type form. No
/// methods of its own — everything lives on [`SampleProcessor`] so
/// it can be overridden.
pub trait SampleFilter<T>: SampleProcessor<T, Output = T> {}

impl<T, F> SampleFilter<T> for F where F: SampleProcessor<T, Output = T> {}
