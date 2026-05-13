use crate::traits::{Reset, SampleProcessor};

/// Fixed integer-sample delay line of length `N`.
///
/// Stores up to `N` past samples in a circular buffer. Calling
/// [`process_sample`] returns the sample written `N` calls ago (or `T`'s
/// default for the first `N` calls after [`reset`] / construction).
///
/// `N == 0` collapses to identity.
///
/// [`process_sample`]: SampleProcessor::process_sample
/// [`reset`]: Reset::reset
#[derive(Clone, Copy, Debug)]
pub struct Delay<T, const N: usize> {
    buf: [T; N],
    head: usize,
}

impl<T: Default + Copy, const N: usize> Delay<T, N> {
    /// Construct an empty delay line (state = `T::default()`).
    pub fn new() -> Self {
        Self {
            buf: [T::default(); N],
            head: 0,
        }
    }
}

impl<T: Default + Copy, const N: usize> Default for Delay<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + Copy, const N: usize> Reset for Delay<T, N> {
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.head = 0;
    }
}

impl<T: Copy + Default, const N: usize> SampleProcessor<T> for Delay<T, N> {
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        if N == 0 {
            return input;
        }
        let out = self.buf[self.head];
        self.buf[self.head] = input;
        self.head += 1;
        if self.head == N {
            self.head = 0;
        }
        out
    }
}
