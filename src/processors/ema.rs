use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Exponential moving average.
///
/// Difference equation:
///
/// ```text
///     y[n] = alpha * x[n] + (1 - alpha) * y[n - 1]
/// ```
///
/// `alpha` is in `(0, 1]`. Smaller values give a slower response;
/// `alpha = 1` passes input through unchanged.
#[derive(Clone, Copy, Debug)]
pub struct Ema<T> {
    /// Weight applied to the new input sample.
    pub alpha: T,
    y: T,
}

impl<T> Ema<T>
where
    T: num_traits::Zero + Copy,
{
    /// Build with zero initial state.
    pub fn new(alpha: T) -> Self {
        Self {
            alpha,
            y: T::zero(),
        }
    }

    /// Build with a pre-loaded previous output value.
    pub fn with_state(alpha: T, last_output: T) -> Self {
        Self {
            alpha,
            y: last_output,
        }
    }

    /// Current previous-output state.
    pub fn last_output(&self) -> T
    where
        T: Copy,
    {
        self.y
    }
}

impl<T> Reset for Ema<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        self.y = T::zero();
    }
}

impl<T> Retune<T> for Ema<T> {
    fn retune(&mut self, alpha: T) {
        self.alpha = alpha;
    }
}

impl<T> SampleProcessor<T> for Ema<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let one = T::one();
        self.y = self.alpha * input + (one - self.alpha) * self.y;
        self.y
    }
}

impl<T> SteadyState<T> for Ema<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    fn reset_to_steady_input(&mut self, input: T) {
        self.y = input;
    }
}

impl<T> FiltFiltKernel<T> for Ema<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        6
    }
}
