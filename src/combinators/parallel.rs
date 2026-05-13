use crate::traits::{Reset, SampleProcessor};

/// Parallel composition that emits both processors' outputs as a tuple.
#[derive(Clone, Copy, Debug)]
pub struct Parallel<A, B> {
    /// Upper branch.
    pub a: A,
    /// Lower branch.
    pub b: B,
}

impl<A, B> Parallel<A, B> {
    /// Build a parallel pair.
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A, B> Reset for Parallel<A, B>
where
    A: Reset,
    B: Reset,
{
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}

impl<I, A, B> SampleProcessor<I> for Parallel<A, B>
where
    I: Copy,
    A: SampleProcessor<I>,
    B: SampleProcessor<I>,
{
    type Output = (A::Output, B::Output);

    fn process_sample(&mut self, input: I) -> Self::Output {
        let oa = self.a.process_sample(input);
        let ob = self.b.process_sample(input);
        (oa, ob)
    }
}
