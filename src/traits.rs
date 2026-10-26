//! Core processing traits.

use crate::combinators::{Bypass, Chain, Map, Parallel, Sum, Tap, WetDry};

/// Clear runtime state without changing coefficients or parameters.
///
/// After a reset, the processor behaves as if it had just been constructed.
pub trait Reset {
    /// Clear runtime state.
    fn reset(&mut self);
}

/// Replace a processor's coefficients or parameters at runtime.
pub trait Retune<Coeffs> {
    /// Replace the active coefficient set.
    fn retune(&mut self, coeffs: Coeffs);
}

/// A causal processor producing one output sample per input sample.
///
/// `process_into` and `process_in_place` default to calling
/// `process_sample` in a loop; implementations may override them.
pub trait SampleProcessor<I>: Reset {
    /// Sample produced for each input sample.
    type Output;

    /// Process one input sample.
    fn process_sample(&mut self, input: I) -> Self::Output;

    /// Process `input` into `output`.
    ///
    /// # Panics
    ///
    /// Panics if `input.len() != output.len()`.
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

    /// Process `buffer` in place.
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

/// A processor that needs the whole finite signal at once, and may be
/// non-causal (e.g. zero-phase filtering).
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

/// Processors that can start in the steady state for a constant input.
///
/// After `reset_to_steady_input(x)`, the processor behaves as if it had
/// already processed an indefinitely long run of `x`.
pub trait SteadyState<T>: SampleProcessor<T, Output = T> + Reset {
    /// Set runtime state to the steady response for a constant input.
    fn reset_to_steady_input(&mut self, input: T);
}

/// Filters usable with [`ForwardBackward`](crate::whole::ForwardBackward).
pub trait FiltFiltKernel<T>: SteadyState<T> {
    /// Default edge-padding length per side, following SciPy's `filtfilt`.
    fn filtfilt_pad_len(&self) -> usize;
}

/// Fluent composition methods, implemented for every [`SampleProcessor`].
///
/// ```
/// use filterkit::{Gain, ProcessorExt, SampleProcessor};
///
/// let mut p = Gain::new(0.5_f32)
///     .then(Gain::new(2.0))
///     .tap(|s: &f32| println!("{}", s));
///
/// let y = p.process_sample(0.7);
/// ```
pub trait ProcessorExt<I>: SampleProcessor<I> + Sized {
    /// Feed this processor's output into `next`.
    fn then<B>(self, next: B) -> Chain<Self, B>
    where
        B: SampleProcessor<Self::Output>,
    {
        Chain::new(self, next)
    }

    /// Run `self` and `other` on the same input; output both as a tuple.
    fn parallel<B>(self, other: B) -> Parallel<Self, B>
    where
        B: SampleProcessor<I>,
        I: Copy,
    {
        Parallel::new(self, other)
    }

    /// Run `self` and `other` on the same input; output their sum.
    fn sum<B>(self, other: B) -> Sum<Self, B>
    where
        B: SampleProcessor<I, Output = Self::Output>,
        I: Copy,
    {
        Sum::new(self, other)
    }

    /// Wrap in a [`Bypass`] that can be toggled at runtime.
    fn bypass(self) -> Bypass<Self>
    where
        Self: SampleProcessor<I, Output = I>,
    {
        Bypass::new(self)
    }

    /// Mix dry input with processed output.
    fn wet_dry(self, wet: Self::Output) -> WetDry<Self, Self::Output>
    where
        Self: SampleProcessor<I, Output = I>,
    {
        WetDry::new(self, wet)
    }

    /// Apply `f` to each output sample.
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        F: FnMut(Self::Output) -> O,
    {
        Map::new(self, f)
    }

    /// Call `f` on each output sample without changing it.
    fn tap<F>(self, f: F) -> Tap<Self, F>
    where
        F: FnMut(&Self::Output),
        Self::Output: Copy,
    {
        Tap::new(self, f)
    }
}

impl<I, P> ProcessorExt<I> for P where P: SampleProcessor<I> {}
