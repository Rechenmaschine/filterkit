use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Coefficients for a general first-order IIR.
///
/// The implemented difference equation is
///
/// ```text
///     y[n] = b0*x[n] + b1*x[n-1] - a1*y[n-1]
/// ```
///
/// The denominator's leading `a0` is implicit and equal to one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstOrderCoeffs<T> {
    /// Current-input feed-forward coefficient.
    pub b0: T,
    /// Previous-input feed-forward coefficient.
    pub b1: T,
    /// Previous-output feedback coefficient, with the sign convention
    /// shown in the difference equation above.
    pub a1: T,
}

impl<T> FirstOrderCoeffs<T> {
    /// Construct from normalised coefficients.
    pub const fn new(b0: T, b1: T, a1: T) -> Self {
        Self { b0, b1, a1 }
    }
}

impl<T> FirstOrderCoeffs<T>
where
    T: Copy + core::ops::Div<Output = T>,
{
    /// Construct from an unnormalised four-tuple `(b0, b1, a0, a1)`.
    pub fn from_unnormalised(b0: T, b1: T, a0: T, a1: T) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            a1: a1 / a0,
        }
    }
}

impl<T> FirstOrderCoeffs<T>
where
    T: num_traits::Zero + num_traits::One,
{
    /// Identity transfer function.
    pub fn identity() -> Self {
        Self {
            b0: T::one(),
            b1: T::zero(),
            a1: T::zero(),
        }
    }
}

/// General first-order IIR processor.
#[derive(Clone, Copy, Debug)]
pub struct FirstOrder<T> {
    /// Active coefficient set.
    pub coeffs: FirstOrderCoeffs<T>,
    x1: T,
    y1: T,
}

impl<T> FirstOrder<T>
where
    T: num_traits::Zero + Copy,
{
    /// Build with zero initial state.
    pub fn new(coeffs: FirstOrderCoeffs<T>) -> Self {
        Self {
            coeffs,
            x1: T::zero(),
            y1: T::zero(),
        }
    }

    /// Build with pre-loaded previous input and output.
    pub fn with_state(coeffs: FirstOrderCoeffs<T>, x1: T, y1: T) -> Self {
        Self { coeffs, x1, y1 }
    }
}

impl<T> Reset for FirstOrder<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        self.x1 = T::zero();
        self.y1 = T::zero();
    }
}

impl<T> Retune<FirstOrderCoeffs<T>> for FirstOrder<T> {
    fn retune(&mut self, coeffs: FirstOrderCoeffs<T>) {
        self.coeffs = coeffs;
    }
}

impl<T> SampleProcessor<T> for FirstOrder<T>
where
    T: Copy
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let y = self.coeffs.b0 * input + self.coeffs.b1 * self.x1 - self.coeffs.a1 * self.y1;
        self.x1 = input;
        self.y1 = y;
        y
    }
}

impl<T> SteadyState<T> for FirstOrder<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn reset_to_steady_input(&mut self, input: T) {
        let one = T::one();
        let steady = (self.coeffs.b0 + self.coeffs.b1) * input / (one + self.coeffs.a1);
        self.x1 = input;
        self.y1 = steady;
    }
}

impl<T> FiltFiltKernel<T> for FirstOrder<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        6
    }
}
