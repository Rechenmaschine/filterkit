use crate::traits::{Reset, SampleProcessor};

/// Series composition: `a -> b`.
///
/// Output of `a` is fed as input to `b`. The two processors may have
/// different sample types — only `A::Output` and `B`'s input type must
/// match.
#[derive(Clone, Copy, Debug)]
pub struct Chain<A, B> {
    /// First processor in the chain.
    pub a: A,
    /// Second processor.
    pub b: B,
}

impl<A, B> Chain<A, B> {
    /// Build a chain.
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Reset for Chain<A, B>
where
    A: Reset,
    B: Reset,
{
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}

impl<I, A, B> SampleProcessor<I> for Chain<A, B>
where
    A: SampleProcessor<I>,
    B: SampleProcessor<A::Output>,
{
    type Output = B::Output;

    fn process_sample(&mut self, input: I) -> Self::Output {
        let mid = self.a.process_sample(input);
        self.b.process_sample(mid)
    }
}
