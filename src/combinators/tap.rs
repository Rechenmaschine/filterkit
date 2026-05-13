use core::fmt;

use crate::traits::{Reset, SampleProcessor};

/// Inspect each output sample without altering it.
///
/// Useful for instrumentation, metering, or pulling samples into a
/// queue/oscilloscope.
pub struct Tap<F, Fun> {
    /// Inner processor.
    pub inner: F,
    /// Inspector function.
    pub func: Fun,
}

impl<F: fmt::Debug, Fun> fmt::Debug for Tap<F, Fun> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Tap").field("inner", &self.inner).finish_non_exhaustive()
    }
}

impl<F, Fun> Tap<F, Fun> {
    /// Build a tap.
    pub const fn new(inner: F, func: Fun) -> Self {
        Self { inner, func }
    }
}

impl<F: Reset, Fun> Reset for Tap<F, Fun> {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<I, F, Fun> SampleProcessor<I> for Tap<F, Fun>
where
    F: SampleProcessor<I>,
    F::Output: Copy,
    Fun: FnMut(&F::Output),
{
    type Output = F::Output;

    fn process_sample(&mut self, input: I) -> Self::Output {
        let y = self.inner.process_sample(input);
        (self.func)(&y);
        y
    }
}
