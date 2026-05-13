use crate::coeffs::SosCoeffs;
use crate::processors::biquad::BiquadState;
use crate::traits::{Reset, Retune, SampleProcessor};

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
