#[cfg(feature = "alloc")]
use alloc::boxed::Box;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::traits::{Prepare, ProcessSpec, Reset, StreamProcessor, StreamStatus};

/// Polyphase rational resampler at ratio `L / M`.
///
/// Conceptually:
///
/// 1. Zero-stuff input by factor `L` (insert `L - 1` zeros between samples).
/// 2. Lowpass filter with the prototype FIR.
/// 3. Decimate by `M`.
///
/// The prototype FIR is reorganised into `L` polyphase subfilters of
/// length `ceil(num_taps / L)`. For each output sample we run exactly
/// one subfilter against the input delay line, choosing the subfilter
/// by the current phase.
///
/// Requires the `alloc` feature: the polyphase tables and input delay
/// line are heap-backed since their size depends on the runtime FIR
/// length and on `L`. For const-sized stages, build the FIR directly
/// and use [`Decimator`] / [`Interpolator`] instead.
///
/// [`Decimator`]: super::Decimator
/// [`Interpolator`]: super::Interpolator
#[cfg(feature = "alloc")]
#[derive(Clone, Debug)]
pub struct PolyphaseResampler<T> {
    /// L (interpolation factor).
    up: usize,
    /// M (decimation factor).
    down: usize,
    /// Subfilters: `phases[i]` has length `sub_len`. Row-major flat
    /// storage with stride `sub_len`.
    phases: Box<[T]>,
    sub_len: usize,
    /// Input delay line, length `sub_len`.
    buf: Box<[T]>,
    head: usize,
    /// Phase counter in `0..L`.
    phase: usize,
}

#[cfg(feature = "alloc")]
impl<T> PolyphaseResampler<T>
where
    T: Default + Copy + num_traits::Zero,
{
    /// Build a polyphase resampler.
    ///
    /// `taps` is the prototype FIR.
    ///
    /// # Panics
    ///
    /// Panics if `up == 0`, `down == 0`, or `taps.is_empty()`. Also
    /// panics if `taps.len() < up`, because some polyphase sub-filters
    /// would have no coefficients.
    pub fn new(taps: &[T], up: usize, down: usize) -> Self {
        assert!(up >= 1 && down >= 1, "up/down must be >= 1");
        assert!(!taps.is_empty(), "taps slice must be non-empty");
        assert!(
            taps.len() >= up,
            "taps.len() = {} is less than up = {}; polyphase decomposition would have empty sub-filters",
            taps.len(),
            up,
        );

        // Subfilter length: ceil(len(taps) / up).
        let sub_len = taps.len().div_ceil(up.max(1));

        let mut phases: Vec<T> = Vec::with_capacity(up * sub_len);
        phases.resize(up * sub_len, T::zero());
        // Polyphase decomposition: phase i takes taps[i], taps[i + L], …
        for i in 0..up {
            for k in 0..sub_len {
                let src = i + k * up;
                if src < taps.len() {
                    phases[i * sub_len + k] = taps[src];
                }
            }
        }

        Self {
            up,
            down,
            phases: phases.into_boxed_slice(),
            sub_len,
            buf: alloc::vec![T::default(); sub_len.max(1)].into_boxed_slice(),
            head: 0,
            phase: 0,
        }
    }

    /// Interpolation factor `L`.
    pub fn up(&self) -> usize {
        self.up
    }

    /// Decimation factor `M`.
    pub fn down(&self) -> usize {
        self.down
    }
}

/// Error returned by [`PolyphaseResampler::prepare`].
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolyphaseResamplerPrepareError {
    /// Reported `max_block_len` of zero. No work would be possible.
    EmptyBlock,
}

#[cfg(feature = "alloc")]
impl<T, S> Prepare<S> for PolyphaseResampler<T>
where
    T: Default + Copy,
{
    type Error = PolyphaseResamplerPrepareError;

    fn prepare(&mut self, spec: ProcessSpec<S>) -> Result<(), Self::Error> {
        if spec.max_block_len == 0 {
            return Err(PolyphaseResamplerPrepareError::EmptyBlock);
        }
        self.reset();
        Ok(())
    }
}

#[cfg(feature = "alloc")]
impl<T> Reset for PolyphaseResampler<T>
where
    T: Default + Copy,
{
    fn reset(&mut self) {
        for s in self.buf.iter_mut() {
            *s = T::default();
        }
        self.head = 0;
        self.phase = 0;
    }
}

#[cfg(feature = "alloc")]
impl<T> StreamProcessor<T> for PolyphaseResampler<T>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    type Output = T;

    fn process_stream(&mut self, input: &[T], output: &mut [Self::Output]) -> StreamStatus {
        let mut consumed = 0usize;
        let mut produced = 0usize;

        if self.up == 0 || self.down == 0 || self.sub_len == 0 {
            return StreamStatus { consumed, produced };
        }

        // Drain pending phases that don't need a new input.
        while produced < output.len() && self.phase >= self.up {
            self.phase -= self.up;
            output[produced] = self.eval(self.phase);
            produced += 1;
            self.phase += self.down;
        }

        for &x in input {
            self.buf[self.head] = x;
            self.head = (self.head + 1) % self.sub_len;
            consumed += 1;

            // Emit all outputs whose phase falls in `0..up` for this new
            // input. After each emission, phase advances by `down`.
            while produced < output.len() && self.phase < self.up {
                output[produced] = self.eval(self.phase);
                produced += 1;
                self.phase += self.down;
            }

            // Bring phase back into the next input's window.
            if self.phase >= self.up {
                self.phase -= self.up;
            }

            if produced >= output.len() {
                break;
            }
        }

        StreamStatus { consumed, produced }
    }

    fn input_needed(&self, output_len: usize) -> Option<usize> {
        if self.up == 0 {
            return None;
        }
        // Rough upper bound: each output needs `down` advances of phase,
        // and a new input is required every `up` units of phase.
        Some(output_len.saturating_mul(self.down).div_ceil(self.up))
    }
}

#[cfg(feature = "alloc")]
impl<T> PolyphaseResampler<T>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    /// Dot the input delay line against polyphase row `phase`.
    fn eval(&self, phase: usize) -> T {
        let row = &self.phases[phase * self.sub_len..(phase + 1) * self.sub_len];
        let mut acc = T::zero();
        let n = self.sub_len;
        let mut idx = if self.head == 0 { n - 1 } else { self.head - 1 };
        for k in 0..n {
            acc = acc + row[k] * self.buf[idx];
            idx = if idx == 0 { n - 1 } else { idx - 1 };
        }
        acc
    }
}
