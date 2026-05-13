use alloc::boxed::Box;
use alloc::vec;

use super::lms::LmsScalar;
use crate::traits::{AdaptiveProcessor, Reset};

/// Numeric requirements for [`Nlms`]. Adds `Div` to [`LmsScalar`].
pub trait NlmsScalar: LmsScalar + core::ops::Div<Output = Self> {}

impl<T> NlmsScalar for T where T: LmsScalar + core::ops::Div<Output = T> {}

/// Normalised LMS.
///
/// Like [`super::Lms`] but the effective step size is divided by the
/// instantaneous input power `x^T x + epsilon`, which keeps adaptation
/// stable as input level varies.
///
/// # Call-ordering contract
///
/// Same as [`super::Lms`]: call `process_sample` before `adapt` for each
/// step, or use `process_adapt` from
/// [`crate::AdaptiveProcessor::process_adapt`].
#[derive(Clone, Debug)]
pub struct Nlms<T> {
    /// Weight vector `w[0..N]`.
    pub w: Box<[T]>,
    /// Step size.
    pub mu: T,
    /// Regularisation constant added to the denominator. Prevents
    /// division by zero on silent inputs.
    pub eps: T,
    buf: Box<[T]>,
    head: usize,
}

impl<T> Nlms<T>
where
    T: NlmsScalar,
{
    /// Build an NLMS filter.
    pub fn new(n: usize, mu: T, eps: T) -> Self {
        let zero = T::zero();
        Self {
            w: vec![zero; n].into_boxed_slice(),
            mu,
            eps,
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

impl<T> Reset for Nlms<T>
where
    T: NlmsScalar,
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

impl<T> AdaptiveProcessor<T> for Nlms<T>
where
    T: NlmsScalar,
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

        // Compute input power x^T x.
        let mut power = T::zero();
        for k in 0..n {
            power = power + self.buf[k] * self.buf[k];
        }

        let scale = self.mu * err / (power + self.eps);

        let mut idx = if self.head == 0 { n - 1 } else { self.head - 1 };
        for k in 0..n {
            self.w[k] = self.w[k] + scale * self.buf[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        err
    }
}
