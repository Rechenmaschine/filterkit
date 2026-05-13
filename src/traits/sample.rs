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
/// [`Output`]: SampleProcessor::Output
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
}

/// A [`SampleProcessor`] whose input and output sample types coincide.
///
/// Provided as a marker plus an `in_place` helper. Implemented
/// automatically for every `SampleProcessor<T, Output = T>`.
pub trait SampleFilter<T>: SampleProcessor<T, Output = T> {
    /// Process samples in-place. Default implementation calls
    /// [`SampleProcessor::process_sample`] over each element.
    fn process_in_place(&mut self, buffer: &mut [T])
    where
        T: Copy,
    {
        for x in buffer.iter_mut() {
            *x = self.process_sample(*x);
        }
    }
}

impl<T, F> SampleFilter<T> for F where F: SampleProcessor<T, Output = T> {}
