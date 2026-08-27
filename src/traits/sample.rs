use super::Reset;

/// A causal, same-rate processor that consumes one input sample and
/// produces one output sample.
///
/// The associated [`Output`] type lets processors compose without forcing
/// input and output to match.
///
/// # Batch methods
///
/// Two default methods provide batch processing on top of
/// [`process_sample`]. Concrete kernels that can do block-rate work, such
/// as SIMD or FFT processing, can override them.
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

    /// Process one slice into another of the same length. The default
    /// implementation calls [`process_sample`] in a loop.
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
    /// [`SampleProcessor::process_sample`] over each element; concrete kernels are free
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
