use alloc::boxed::Box;
use alloc::vec;

use crate::traits::{AdaptiveProcessor, Reset};

/// Numeric requirements for [`Lms`] coefficient/sample types.
pub trait LmsScalar:
    Copy
    + num_traits::Zero
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
{
}

impl<T> LmsScalar for T where
    T: Copy
        + num_traits::Zero
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Mul<Output = T>
{
}

/// Least-Mean-Squares adaptive FIR.
///
/// Maintains a length-`N` weight vector and a length-`N` delay line of
/// past inputs. On each [`adapt`] call, weights are pushed in the
/// direction of the error scaled by step size `mu` and the most recent
/// input sample.
///
/// # Call-ordering contract
///
/// `adapt` reads the most recent sample from the internal delay line,
/// which was written by [`process_sample`]. Callers must invoke
/// `process_sample` for the current input *before* calling `adapt` for
/// that same step. Using the bundled [`process_adapt`] helper enforces
/// this ordering for you.
///
/// Requires the `alloc` feature; weights and delay line are heap-backed
/// because LMS lengths typically vary with the application.
///
/// [`adapt`]: AdaptiveProcessor::adapt
/// [`process_sample`]: AdaptiveProcessor::process_sample
/// [`process_adapt`]: AdaptiveProcessor::process_adapt
#[derive(Clone, Debug)]
pub struct Lms<T> {
    /// Weight vector `w[0..N]`.
    pub w: Box<[T]>,
    /// Step size.
    pub mu: T,
    buf: Box<[T]>,
    head: usize,
}

impl<T> Lms<T>
where
    T: LmsScalar,
{
    /// Build an LMS filter of length `n` with the given step size.
    pub fn new(n: usize, mu: T) -> Self {
        let zero = T::zero();
        Self {
            w: vec![zero; n].into_boxed_slice(),
            mu,
            buf: vec![zero; n.max(1)].into_boxed_slice(),
            head: 0,
        }
    }

    /// Number of taps.
    pub fn len(&self) -> usize {
        self.w.len()
    }

    /// `true` when there are no taps.
    pub fn is_empty(&self) -> bool {
        self.w.is_empty()
    }
}

impl<T> Reset for Lms<T>
where
    T: LmsScalar,
{
    fn reset(&mut self) {
        let zero = T::zero();
        for w in self.w.iter_mut() {
            *w = zero;
        }
        for s in self.buf.iter_mut() {
            *s = zero;
        }
        self.head = 0;
    }
}

impl<T> AdaptiveProcessor<T> for Lms<T>
where
    T: LmsScalar,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let n = self.w.len();
        if n == 0 {
            return T::zero();
        }
        self.buf[self.head] = input;

        let mut acc = T::zero();
        let mut idx = self.head;
        for k in 0..n {
            acc = acc + self.w[k] * self.buf[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }

        self.head = (self.head + 1) % n;
        acc
    }

    fn adapt(&mut self, desired: Self::Output, output: Self::Output) -> Self::Output {
        let err = desired - output;
        let n = self.w.len();
        if n == 0 {
            return err;
        }

        // The most recent x sample is at index `head - 1`.
        let mut idx = if self.head == 0 { n - 1 } else { self.head - 1 };
        let scale = self.mu * err;
        for k in 0..n {
            self.w[k] = self.w[k] + scale * self.buf[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        err
    }
}
