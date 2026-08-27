use crate::coeffs::BiquadCoeffs;
use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Direct-form-II transposed state for one biquad.
///
/// The DF2T topology uses two state values for a second-order section.
#[derive(Clone, Copy, Debug, Default)]
pub struct BiquadState<T> {
    /// `s1` from the DF2T diagram.
    pub s1: T,
    /// `s2` from the DF2T diagram.
    pub s2: T,
}

impl<T: num_traits::Zero> BiquadState<T> {
    /// Zeroed state.
    pub fn zero() -> Self {
        Self {
            s1: T::zero(),
            s2: T::zero(),
        }
    }
}

/// Single biquad (second-order section) running in direct-form-II
/// transposed.
#[derive(Clone, Copy, Debug)]
pub struct Biquad<T> {
    /// Active coefficient set.
    pub coeffs: BiquadCoeffs<T>,
    state: BiquadState<T>,
}

impl<T> Biquad<T>
where
    T: num_traits::Zero + Copy,
{
    /// Build a biquad from coefficients with zeroed state.
    pub fn new(coeffs: BiquadCoeffs<T>) -> Self {
        Self {
            coeffs,
            state: BiquadState::zero(),
        }
    }
}

impl<T> Reset for Biquad<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        self.state = BiquadState::zero();
    }
}

impl<T> Retune<BiquadCoeffs<T>> for Biquad<T> {
    fn retune(&mut self, coeffs: BiquadCoeffs<T>) {
        self.coeffs = coeffs;
    }
}

impl<T> SampleProcessor<T> for Biquad<T>
where
    T: Copy
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        // DF2T:
        //   y    = b0*x + s1
        //   s1'  = b1*x - a1*y + s2
        //   s2'  = b2*x - a2*y
        let y = self.coeffs.b0 * input + self.state.s1;
        let s1_next = self.coeffs.b1 * input - self.coeffs.a1 * y + self.state.s2;
        let s2_next = self.coeffs.b2 * input - self.coeffs.a2 * y;
        self.state.s1 = s1_next;
        self.state.s2 = s2_next;
        y
    }
}

impl<T> SteadyState<T> for Biquad<T>
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
        let c = self.coeffs;
        let numerator = c.b0 + c.b1 + c.b2;
        let denominator = one + c.a1 + c.a2;
        let steady = numerator * input / denominator;
        self.state.s1 = steady - c.b0 * input;
        self.state.s2 = c.b2 * input - c.a2 * steady;
    }
}

impl<T> FiltFiltKernel<T> for Biquad<T>
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
        9
    }
}
