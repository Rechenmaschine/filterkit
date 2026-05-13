use crate::traits::{Reset, SampleProcessor};

/// Wet/dry mixer.
///
/// Computes `(1 - wet) * dry + wet * processed`. `wet` typically lies
/// in `[0, 1]` but is not clamped — values outside the range produce
/// over/under-driven blends, which is sometimes intentional.
#[derive(Clone, Copy, Debug)]
pub struct WetDry<F, T> {
    /// Inner processor.
    pub inner: F,
    /// Wet mix coefficient.
    pub wet: T,
}

impl<F, T> WetDry<F, T> {
    /// Build with the given wet mix.
    pub const fn new(inner: F, wet: T) -> Self {
        Self { inner, wet }
    }
}

impl<F: Reset, T> Reset for WetDry<F, T> {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<T, F> SampleProcessor<T> for WetDry<F, T>
where
    T: Copy
        + num_traits::One
        + core::ops::Sub<Output = T>
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>,
    F: SampleProcessor<T, Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let processed = self.inner.process_sample(input);
        let one = T::one();
        (one - self.wet) * input + self.wet * processed
    }
}
