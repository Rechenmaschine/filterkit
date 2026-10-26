use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Heap-backed FIR filter of arbitrary length.
///
/// Same algorithm as [`crate::processors::Fir`], but the tap count is
/// only known at runtime. Useful when filters are loaded from files,
/// resized for tuning, or held generically alongside other dynamic
/// processors. Requires the `alloc` feature.
#[derive(Clone, Debug)]
pub struct FirDyn<T> {
    b: Box<[T]>,
    buf: Vec<T>,
    head: usize,
}

impl<T: Default + Copy> FirDyn<T> {
    /// Build from a slice of taps.
    pub fn new(taps: &[T]) -> Self {
        let len = taps.len();
        Self {
            b: taps.to_vec().into_boxed_slice(),
            buf: vec![T::default(); len],
            head: 0,
        }
    }

    /// Tap values.
    pub fn taps(&self) -> &[T] {
        &self.b
    }

    /// Number of taps.
    pub fn len(&self) -> usize {
        self.b.len()
    }

    /// `true` when no taps are configured.
    pub fn is_empty(&self) -> bool {
        self.b.is_empty()
    }
}

impl<T: Default + Copy> Reset for FirDyn<T> {
    fn reset(&mut self) {
        for s in self.buf.iter_mut() {
            *s = T::default();
        }
        self.head = 0;
    }
}

impl<T: Default + Copy> Retune<Vec<T>> for FirDyn<T> {
    /// Keeps history if the tap count is unchanged; otherwise clears it.
    fn retune(&mut self, coeffs: Vec<T>) {
        if coeffs.len() != self.b.len() {
            self.buf = vec![T::default(); coeffs.len()];
            self.head = 0;
        }
        self.b = coeffs.into_boxed_slice();
    }
}

impl<T> SampleProcessor<T> for FirDyn<T>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let n = self.b.len();
        if n == 0 {
            return T::zero();
        }
        self.buf[self.head] = input;

        let mut acc = T::zero();
        let mut idx = self.head;
        for k in 0..n {
            acc = acc + self.b[k] * self.buf[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }

        self.head += 1;
        if self.head == n {
            self.head = 0;
        }
        acc
    }
}

impl<T> SteadyState<T> for FirDyn<T>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    fn reset_to_steady_input(&mut self, input: T) {
        for s in self.buf.iter_mut() {
            *s = input;
        }
        self.head = 0;
    }
}

impl<T> FiltFiltKernel<T> for FirDyn<T>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        3 * self.b.len()
    }
}
