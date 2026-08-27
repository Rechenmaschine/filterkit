use crate::coeffs::FirCoeffs;
use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Per-instance state for a length-`N` FIR filter.
///
/// Holds the last `N` input samples as a circular buffer. Kept separate
/// from [`FirCoeffs`] so multiple voices can share one coefficient
/// block.
#[derive(Clone, Copy, Debug)]
pub struct FirState<T, const N: usize> {
    buf: [T; N],
    head: usize,
}

impl<T: Default + Copy, const N: usize> FirState<T, N> {
    /// Empty state.
    pub fn new() -> Self {
        Self {
            buf: [T::default(); N],
            head: 0,
        }
    }
}

impl<T: Default + Copy, const N: usize> Default for FirState<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Length-`N` FIR filter executing on owned [`FirCoeffs`] and
/// [`FirState`].
///
/// Direct realisation: each sample multiplies the last `N` inputs by the
/// `b` taps. For a sparse FIR you might prefer a dedicated kernel; for
/// most short-to-mid-length filters the direct form is fine.
#[derive(Clone, Copy, Debug)]
pub struct Fir<T, const N: usize> {
    /// Coefficient block.
    pub coeffs: FirCoeffs<T, N>,
    state: FirState<T, N>,
}

impl<T: Default + Copy, const N: usize> Fir<T, N> {
    /// Build a fresh FIR from its taps.
    pub fn new(coeffs: FirCoeffs<T, N>) -> Self {
        Self {
            coeffs,
            state: FirState::default(),
        }
    }

    /// Build a FIR with a pre-loaded delay line.
    ///
    /// `history[0]` is treated as the most recent past sample
    /// (`x[n-1]`), `history[N-1]` as the oldest (`x[n-N]`). Useful for
    /// stitching together processing of contiguous blocks without
    /// introducing a discontinuity at the join.
    pub fn with_history(coeffs: FirCoeffs<T, N>, history: [T; N]) -> Self {
        // Store history in the circular layout expected by `process_sample`.
        let mut buf = [T::default(); N];
        if N > 0 {
            for k in 0..N {
                buf[(N - 1 - k) % N] = history[k];
            }
        }
        Self {
            coeffs,
            state: FirState { buf, head: 0 },
        }
    }

    /// Number of taps.
    pub const fn len(&self) -> usize {
        N
    }

    /// True when there are no taps (`N == 0`).
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<T: Default + Copy, const N: usize> Reset for Fir<T, N> {
    fn reset(&mut self) {
        self.state = FirState::default();
    }
}

impl<T, const N: usize> Retune<FirCoeffs<T, N>> for Fir<T, N> {
    fn retune(&mut self, coeffs: FirCoeffs<T, N>) {
        self.coeffs = coeffs;
    }
}

impl<T, const N: usize> SampleProcessor<T> for Fir<T, N>
where
    T: Copy + Default + core::ops::Mul<Output = T> + core::ops::Add<Output = T> + num_traits::Zero,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        if N == 0 {
            return T::zero();
        }
        self.state.buf[self.state.head] = input;

        let mut acc = T::zero();
        let mut idx = self.state.head;
        for k in 0..N {
            acc = acc + self.coeffs.b[k] * self.state.buf[idx];
            idx = if idx == 0 { N - 1 } else { idx - 1 };
        }

        self.state.head += 1;
        if self.state.head == N {
            self.state.head = 0;
        }

        acc
    }
}

impl<T, const N: usize> SteadyState<T> for Fir<T, N>
where
    T: Copy + Default + core::ops::Mul<Output = T> + core::ops::Add<Output = T> + num_traits::Zero,
{
    fn reset_to_steady_input(&mut self, input: T) {
        self.state.buf = [input; N];
        self.state.head = 0;
    }
}

impl<T, const N: usize> FiltFiltKernel<T> for Fir<T, N>
where
    T: Copy + Default + core::ops::Mul<Output = T> + core::ops::Add<Output = T> + num_traits::Zero,
{
    fn filtfilt_pad_len(&self) -> usize {
        3 * N
    }
}
