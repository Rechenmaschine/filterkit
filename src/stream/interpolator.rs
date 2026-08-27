use crate::traits::{Reset, StreamProcessor, StreamStatus};

/// Integer-factor interpolator with an FIR reconstruction prototype.
///
/// Interpolates by `L`: produces `L` output samples for every input.
/// Internally a zero-stuff + lowpass; the FIR `taps` provided by the
/// caller should be an `L`-times-upsampled lowpass with passband at
/// `fs / 2L`.
///
#[derive(Debug)]
pub struct Interpolator<'taps, T, const N: usize> {
    /// FIR taps applied after zero stuffing.
    pub taps: &'taps [T; N],
    buf: [T; N],
    head: usize,
    /// Index 0..L of the next zero-stuffed sample we will emit.
    phase: usize,
    factor: usize,
}

impl<'taps, T, const N: usize> Interpolator<'taps, T, N>
where
    T: Default + Copy,
{
    /// Build an interpolator.
    ///
    /// # Panics
    ///
    /// Panics if `factor == 0`.
    pub fn new(taps: &'taps [T; N], factor: usize) -> Self {
        assert!(factor >= 1, "interpolation factor must be >= 1");
        Self {
            taps,
            buf: [T::default(); N],
            head: 0,
            phase: 0,
            factor,
        }
    }

    /// Interpolation factor `L`.
    pub fn factor(&self) -> usize {
        self.factor
    }
}

impl<'taps, T, const N: usize> Reset for Interpolator<'taps, T, N>
where
    T: Default + Copy,
{
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.head = 0;
        self.phase = 0;
    }
}

impl<'taps, T, const N: usize> StreamProcessor<T> for Interpolator<'taps, T, N>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    type Output = T;

    fn process_stream(&mut self, input: &[T], output: &mut [Self::Output]) -> StreamStatus {
        let mut consumed = 0usize;
        let mut produced = 0usize;

        let l = self.factor;
        if N == 0 || l == 0 {
            return StreamStatus { consumed, produced };
        }

        // Finish phases left by a previous input.
        while self.phase != 0 && produced < output.len() {
            output[produced] = self.compute_phase();
            produced += 1;
            self.phase += 1;
            if self.phase >= l {
                self.phase = 0;
                break;
            }
        }

        for &x in input {
            if produced >= output.len() {
                break;
            }
            // Insert the input and emit its L phases.
            self.buf[self.head] = x;
            self.head = (self.head + 1) % N;
            consumed += 1;

            output[produced] = self.compute_phase();
            produced += 1;
            self.phase = 1;

            while self.phase < l && produced < output.len() {
                self.buf[self.head] = T::zero();
                self.head = (self.head + 1) % N;
                output[produced] = self.compute_phase();
                produced += 1;
                self.phase += 1;
            }

            if self.phase >= l {
                self.phase = 0;
            } else {
                // Resume the remaining phases on the next call.
                break;
            }
        }

        StreamStatus { consumed, produced }
    }

    fn input_needed(&self, output_len: usize) -> Option<usize> {
        let l = self.factor.max(1);
        Some(output_len.div_ceil(l))
    }
}

impl<'taps, T, const N: usize> Interpolator<'taps, T, N>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    /// FIR dot product at the current head position.
    fn compute_phase(&self) -> T {
        let mut acc = T::zero();
        // head points one *past* the newest sample; step back N times.
        let mut idx = if self.head == 0 { N - 1 } else { self.head - 1 };
        for k in 0..N {
            acc = acc + self.taps[k] * self.buf[idx];
            idx = if idx == 0 { N - 1 } else { idx - 1 };
        }
        acc
    }
}
