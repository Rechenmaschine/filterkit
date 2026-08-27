use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Stateless multiplicative gain.
///
/// A [`SampleProcessor`] that multiplies every input by a fixed value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gain<T> {
    /// Multiplier applied to every sample.
    pub value: T,
}

impl<T> Gain<T> {
    /// Construct a gain.
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: num_traits::One> Gain<T> {
    /// Unity gain.
    pub fn unity() -> Self {
        Self { value: T::one() }
    }
}

impl<T> Reset for Gain<T> {
    fn reset(&mut self) {}
}

impl<T> Retune<T> for Gain<T> {
    fn retune(&mut self, value: T) {
        self.value = value;
    }
}

impl<T> SampleProcessor<T> for Gain<T>
where
    T: Copy + core::ops::Mul<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        input * self.value
    }
}

impl<T> SteadyState<T> for Gain<T>
where
    T: Copy + core::ops::Mul<Output = T>,
{
    fn reset_to_steady_input(&mut self, _input: T) {
        self.reset();
    }
}

impl<T> FiltFiltKernel<T> for Gain<T>
where
    T: Copy + core::ops::Mul<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        0
    }
}
