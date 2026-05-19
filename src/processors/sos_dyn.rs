use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::coeffs::BiquadCoeffs;
use crate::processors::biquad::BiquadState;
use crate::traits::{
    FiltFiltKernel, Prepare, ProcessSpec, Reset, Retune, SampleProcessor, SteadyState,
};

/// Heap-backed SOS cascade of arbitrary length.
///
/// Mirrors [`crate::processors::SosCascade`] but the section count is
/// not const. Requires the `alloc` feature.
#[derive(Clone, Debug)]
pub struct SosDyn<T> {
    /// Active sections.
    pub sections: Box<[BiquadCoeffs<T>]>,
    states: Vec<BiquadState<T>>,
}

impl<T> SosDyn<T>
where
    T: num_traits::Zero + Copy,
{
    /// Build from a slice of biquad sections.
    pub fn new(sections: &[BiquadCoeffs<T>]) -> Self {
        Self {
            sections: sections.to_vec().into_boxed_slice(),
            states: vec![BiquadState::zero(); sections.len()],
        }
    }

    /// Number of sections.
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    /// `true` when no sections are configured.
    pub fn is_empty(&self) -> bool {
        self.sections.is_empty()
    }
}

/// Error returned by [`SosDyn::prepare`] when the spec is unworkable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SosDynPrepareError;

impl<T, S> Prepare<S> for SosDyn<T>
where
    T: num_traits::Zero + Copy,
{
    type Error = SosDynPrepareError;

    fn prepare(&mut self, _spec: ProcessSpec<S>) -> Result<(), Self::Error> {
        self.reset();
        Ok(())
    }
}

impl<T> Reset for SosDyn<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        for s in self.states.iter_mut() {
            *s = BiquadState::zero();
        }
    }
}

impl<T> Retune<Vec<BiquadCoeffs<T>>> for SosDyn<T>
where
    T: num_traits::Zero + Copy,
{
    /// Replace the active sections. Always resets per-section state so
    /// a retune cannot bleed previous filter history into the new
    /// cascade.
    fn retune(&mut self, coeffs: Vec<BiquadCoeffs<T>>) {
        let new_len = coeffs.len();
        self.sections = coeffs.into_boxed_slice();
        self.states = vec![BiquadState::zero(); new_len];
    }
}

impl<T> SampleProcessor<T> for SosDyn<T>
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
        for i in 0..self.sections.len() {
            let c = &self.sections[i];
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

impl<T> SteadyState<T> for SosDyn<T>
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

        for i in 0..self.sections.len() {
            let c = self.sections[i];
            let numerator = c.b0 + c.b1 + c.b2;
            let denominator = one + c.a1 + c.a2;
            let steady = numerator * section_input / denominator;
            self.states[i].s1 = steady - c.b0 * section_input;
            self.states[i].s2 = c.b2 * section_input - c.a2 * steady;
            section_input = steady;
        }
    }
}

impl<T> FiltFiltKernel<T> for SosDyn<T>
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
        let n = self.sections.len();
        if n == 0 {
            return 0;
        }

        let mut zeros_at_origin = 0;
        let mut poles_at_origin = 0;
        for section in self.sections.iter().copied() {
            if section.b2 == T::zero() {
                zeros_at_origin += 1;
            }
            if section.a2 == T::zero() {
                poles_at_origin += 1;
            }
        }

        3 * (2 * n + 1 - zeros_at_origin.min(poles_at_origin))
    }
}
