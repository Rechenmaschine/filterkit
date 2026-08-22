use super::SampleProcessor;
use crate::combinators::{Bypass, Chain, Map, Parallel, Sum, Tap, WetDry};

/// Extension methods for fluently composing [`SampleProcessor`]s.
///
/// Blanket-implemented for every `SampleProcessor`.
///
/// # Example
///
/// ```ignore
/// use filterkit::{ProcessorExt, Gain};
///
/// let mut p = Gain::new(0.5_f32)
///     .then(Gain::new(2.0)) // identity overall
///     .tap(|s: &f32| println!("{}", s));
///
/// let y = p.process_sample(0.7);
/// ```
pub trait ProcessorExt<I>: SampleProcessor<I> + Sized {
    /// Pipe this processor's output into `next`.
    fn then<B>(self, next: B) -> Chain<Self, B>
    where
        B: SampleProcessor<Self::Output>,
    {
        Chain::new(self, next)
    }

    /// Run this processor and `other` on the *same* input, returning
    /// both outputs as a tuple.
    fn parallel<B>(self, other: B) -> Parallel<Self, B>
    where
        B: SampleProcessor<I>,
        I: Copy,
    {
        Parallel::new(self, other)
    }

    /// Run this processor and `other` on the same input and sum their
    /// outputs.
    fn sum<B>(self, other: B) -> Sum<Self, B>
    where
        B: SampleProcessor<I, Output = Self::Output>,
        I: Copy,
    {
        Sum::new(self, other)
    }

    /// Wrap so the processor can be toggled out of the signal path. When
    /// disabled, [`Bypass`] passes input through unchanged.
    fn bypass(self) -> Bypass<Self>
    where
        Self: SampleProcessor<I, Output = I>,
    {
        Bypass::new(self)
    }

    /// Wet/dry mix between input and processed output. Requires the
    /// processor to have the same input and output type.
    fn wet_dry(self, wet: Self::Output) -> WetDry<Self, Self::Output>
    where
        Self: SampleProcessor<I, Output = I>,
    {
        WetDry::new(self, wet)
    }

    /// Apply a function to the output.
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
    {
        Map::new(self, f)
    }

    /// Inspect each output sample without changing it.
    fn tap<F>(self, f: F) -> Tap<Self, F>
    where
        F: FnMut(&Self::Output),
        Self::Output: Copy,
    {
        Tap::new(self, f)
    }
}

impl<I, P> ProcessorExt<I> for P where P: SampleProcessor<I> {}
