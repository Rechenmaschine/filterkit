use crate::coeffs::SosCoeffs;
use crate::processors::biquad::BiquadState;
use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Cascade of `N` biquads in DF2T.
///
/// State is held in `N` [`BiquadState`]s, one per section, so the
/// coefficient block can be shared while the cascade itself remains
/// per-instance.
#[derive(Clone, Copy, Debug)]
pub struct SosCascade<T, const N: usize> {
    /// Active SOS coefficients.
    pub coeffs: SosCoeffs<T, N>,
    states: [BiquadState<T>; N],
}

impl<T, const N: usize> SosCascade<T, N>
where
    T: num_traits::Zero + Copy,
{
    /// Build a cascade with zeroed state for every section.
    pub fn new(coeffs: SosCoeffs<T, N>) -> Self {
        Self {
            coeffs,
            states: [BiquadState::zero(); N],
        }
    }

    /// Number of sections.
    pub const fn len(&self) -> usize {
        N
    }

    /// `true` when there are no sections.
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<T, const N: usize> Reset for SosCascade<T, N>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        self.states = [BiquadState::zero(); N];
    }
}

impl<T, const N: usize> Retune<SosCoeffs<T, N>> for SosCascade<T, N> {
    fn retune(&mut self, coeffs: SosCoeffs<T, N>) {
        self.coeffs = coeffs;
    }
}

impl<T, const N: usize> SampleProcessor<T> for SosCascade<T, N>
where
    T: Copy
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let mut x = input;
        for i in 0..N {
            let c = &self.coeffs.sections[i];
            let s = &mut self.states[i];
            let y = c.b0 * x + s.s1;
            let s1_next = c.b1 * x - c.a1 * y + s.s2;
            let s2_next = c.b2 * x - c.a2 * y;
            s.s1 = s1_next;
            s.s2 = s2_next;
            x = y;
        }
        x
    }
}

impl<T, const N: usize> SteadyState<T> for SosCascade<T, N>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + PartialEq
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn reset_to_steady_input(&mut self, input: T) {
        let one = T::one();
        let mut section_input = input;

        for i in 0..N {
            let c = self.coeffs.sections[i];
            let numerator = c.b0 + c.b1 + c.b2;
            let denominator = one + c.a1 + c.a2;
            let steady = numerator * section_input / denominator;
            self.states[i].s1 = steady - c.b0 * section_input;
            self.states[i].s2 = c.b2 * section_input - c.a2 * steady;
            section_input = steady;
        }
    }
}

impl<T, const N: usize> FiltFiltKernel<T> for SosCascade<T, N>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + PartialEq
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        if N == 0 {
            return 0;
        }

        let mut zeros_at_origin = 0;
        let mut poles_at_origin = 0;
        for section in self.coeffs.sections {
            if section.b2 == T::zero() {
                zeros_at_origin += 1;
            }
            if section.a2 == T::zero() {
                poles_at_origin += 1;
            }
        }

        3 * (2 * N + 1 - zeros_at_origin.min(poles_at_origin))
    }
}
