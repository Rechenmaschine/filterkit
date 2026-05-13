use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use crate::traits::{Prepare, ProcessSpec, Reset, Retune, SampleProcessor};

/// Heap-backed FIR filter of arbitrary length.
///
/// Same algorithm as [`crate::processors::Fir`], but the tap count is
/// only known at runtime. Useful when filters are loaded from files,
/// resized for tuning, or held generically alongside other dynamic
/// processors. Requires the `alloc` feature.
#[derive(Clone, Debug)]
pub struct FirDyn<T> {
    /// Tap values.
    pub b: Box<[T]>,
    buf: Vec<T>,
    head: usize,
}

/// Error returned by [`FirDyn::prepare`] when the spec is unworkable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirDynPrepareError;

impl<T: Default + Copy> FirDyn<T> {
    /// Build from a slice of taps.
    pub fn new(taps: &[T]) -> Self {
        let len = taps.len();
        Self {
            b: taps.to_vec().into_boxed_slice(),
            buf: vec![T::default(); len.max(1)],
            head: 0,
        }
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

impl<T: Default + Copy, S: Copy> Prepare<S> for FirDyn<T> {
    type Error = FirDynPrepareError;

    fn prepare(&mut self, spec: ProcessSpec<S>) -> Result<(), Self::Error> {
        // FirDyn can absorb blocks of any length: it processes
        // sample-by-sample internally and the only per-block buffer it
        // could allocate is `Vec<Output>` which the caller supplies.
        // We just reset state to give a clean stream.
        let _ = spec;
        self.reset();
        Ok(())
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
    /// Replace the active taps. Always resets the delay line so a
    /// retune cannot leak history from the previous filter into the
    /// new one — same-length-different-coeffs retunes used to keep
    /// state silently, which was foot-gunny.
    fn retune(&mut self, coeffs: Vec<T>) {
        let len = coeffs.len();
        self.b = coeffs.into_boxed_slice();
        self.buf = vec![T::default(); len.max(1)];
        self.head = 0;
    }
}

impl<T> SampleProcessor<T> for FirDyn<T>
where
    T: Copy
        + Default
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>,
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
