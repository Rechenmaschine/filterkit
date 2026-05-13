use alloc::vec;
use alloc::vec::Vec;

use crate::traits::{Reset, SampleProcessor, WholeSignalProcessor};

/// Zero-phase forward/backward filtering á la SciPy's `filtfilt`.
///
/// Runs the wrapped causal [`SampleProcessor`] forward over the signal,
/// then backward over the result. The net frequency response is the
/// squared magnitude of the underlying filter with zero phase.
///
/// # ⚠ Boundary behaviour
///
/// This is a **bare double-pass** implementation. It does **not** do
/// any of the padding / initial-condition tricks that SciPy's
/// `filtfilt` performs (reflection padding, Gustafsson method, etc.).
/// Expect non-trivial transients in the first and last ~3× filter
/// time-constant samples of the output. For signals long compared to
/// the filter's settling time this is usually fine; for short signals
/// or transient analysis, pad the input yourself or wait for a more
/// complete implementation.
///
/// Requires the `alloc` feature.
#[derive(Debug)]
pub struct ForwardBackward<F> {
    /// Wrapped causal filter.
    pub filter: F,
}

impl<F> ForwardBackward<F> {
    /// Wrap a causal filter.
    pub const fn new(filter: F) -> Self {
        Self { filter }
    }
}

impl<T, F> WholeSignalProcessor<T> for ForwardBackward<F>
where
    T: Copy + Default,
    F: SampleProcessor<T, Output = T> + Reset,
{
    type Output = T;

    fn process_whole(&mut self, input: &[T], output: &mut [Self::Output]) {
        assert_eq!(
            input.len(),
            output.len(),
            "process_whole: input and output must have equal length",
        );

        // Forward pass into a temporary buffer.
        let mut tmp: Vec<T> = vec![T::default(); input.len()];
        self.filter.reset();
        for (x, y) in input.iter().copied().zip(tmp.iter_mut()) {
            *y = self.filter.process_sample(x);
        }

        // Backward pass: feed the reversed `tmp` through the filter and
        // un-reverse on the way out.
        self.filter.reset();
        let n = tmp.len();
        for i in 0..n {
            let x = tmp[n - 1 - i];
            output[n - 1 - i] = self.filter.process_sample(x);
        }
    }
}
