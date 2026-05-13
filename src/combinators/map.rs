use core::fmt;

use crate::traits::{Reset, SampleProcessor};

/// Post-process each output sample through a function.
///
/// `Map` doesn't introduce its own state. It implements [`Reset`] by
/// forwarding to the wrapped processor.
pub struct Map<F, Fun> {
    /// Inner processor.
    pub inner: F,
    /// Mapping function applied to the inner output.
    pub func: Fun,
}

impl<F: fmt::Debug, Fun> fmt::Debug for Map<F, Fun> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map").field("inner", &self.inner).finish_non_exhaustive()
    }
}

impl<F, Fun> Map<F, Fun> {
    /// Build a map combinator.
    pub const fn new(inner: F, func: Fun) -> Self {
        Self { inner, func }
    }
}

impl<F: Reset, Fun> Reset for Map<F, Fun> {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<I, F, Fun, O> SampleProcessor<I> for Map<F, Fun>
where
    F: SampleProcessor<I>,
    Fun: FnMut(F::Output) -> O,
{
    type Output = O;

    fn process_sample(&mut self, input: I) -> Self::Output {
        let y = self.inner.process_sample(input);
        (self.func)(y)
    }
}
