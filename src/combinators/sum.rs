use crate::traits::{Reset, SampleProcessor};

/// Parallel processors whose outputs are summed.
///
/// Both branches must produce the same output type. Useful for shelf +
/// peak mixers, side-chain blends, multi-band designs that recombine, …
#[derive(Clone, Copy, Debug)]
pub struct Sum<A, B> {
    /// Upper branch.
    pub a: A,
    /// Lower branch.
    pub b: B,
}

impl<A, B> Sum<A, B> {
    /// Build a sum.
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Reset for Sum<A, B>
where
    A: Reset,
    B: Reset,
{
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}

impl<I, A, B> SampleProcessor<I> for Sum<A, B>
where
    I: Copy,
    A: SampleProcessor<I>,
    B: SampleProcessor<I, Output = A::Output>,
    A::Output: core::ops::Add<Output = A::Output>,
{
    type Output = A::Output;

    fn process_sample(&mut self, input: I) -> Self::Output {
        self.a.process_sample(input) + self.b.process_sample(input)
    }
}
