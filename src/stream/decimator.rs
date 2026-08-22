use crate::traits::{Reset, StreamProcessor, StreamStatus};

/// Integer-factor decimator with an FIR anti-aliasing prototype.
///
/// Decimates by `M`: produces one output sample for every `M` consumed.
/// The anti-alias filter is the FIR with taps `taps` (provided by the
/// caller — typically a lowpass at `fs/(2M)`). The decimator keeps an
/// internal phase counter so calls don't have to be aligned to `M`.
///
/// # Phase alignment
///
/// The first output is emitted after the `M`-th input is consumed
/// (i.e. at input index `M - 1`), the next after the `2M`-th, etc.
/// This is intentional — the FIR needs at least `M` samples to have
/// "seen" one decimated period — but it differs from a naive
/// "take every M-th sample starting at 0" reading. Account for the
/// `M - 1` lead-in if you need sample-accurate alignment between
/// undecimated and decimated streams.
///
/// Numerically this is just an FIR + downsampler; the structure here
/// keeps the FIR taps borrowed so the same coefficient table can drive
/// multiple decimators.
#[derive(Debug)]
pub struct Decimator<'taps, T, const N: usize> {
    /// Anti-alias FIR taps (length `N`).
    pub taps: &'taps [T; N],
    buf: [T; N],
    head: usize,
    phase: usize,
    factor: usize,
}

impl<'taps, T, const N: usize> Decimator<'taps, T, N>
where
    T: Default + Copy,
{
    /// Build a decimator. `factor` must be at least 1.
    ///
    /// # Panics
    ///
    /// Panics if `factor == 0`.
    pub fn new(taps: &'taps [T; N], factor: usize) -> Self {
        assert!(factor >= 1, "decimation factor must be >= 1");
        Self {
            taps,
            buf: [T::default(); N],
            head: 0,
            phase: 0,
            factor,
        }
    }

    /// Decimation factor `M`.
    pub fn factor(&self) -> usize {
        self.factor
    }
}

impl<'taps, T, const N: usize> Reset for Decimator<'taps, T, N>
where
    T: Default + Copy,
{
    fn reset(&mut self) {
        self.buf = [T::default(); N];
        self.head = 0;
        self.phase = 0;
    }
}

impl<'taps, T, const N: usize> StreamProcessor<T> for Decimator<'taps, T, N>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    type Output = T;

    fn process_stream(&mut self, input: &[T], output: &mut [Self::Output]) -> StreamStatus {
        let mut consumed = 0usize;
        let mut produced = 0usize;

        for &x in input {
            if N > 0 {
                self.buf[self.head] = x;
            }
            consumed += 1;
            self.phase += 1;

            if self.phase >= self.factor {
                self.phase = 0;

                if produced >= output.len() {
                    return StreamStatus { consumed, produced };
                }

                if N == 0 {
                    output[produced] = T::zero();
                } else {
                    let mut acc = T::zero();
                    let mut idx = self.head;
                    for k in 0..N {
                        acc = acc + self.taps[k] * self.buf[idx];
                        idx = if idx == 0 { N - 1 } else { idx - 1 };
                    }
                    output[produced] = acc;
                }
                produced += 1;
            }

            if N > 0 {
                self.head += 1;
                if self.head == N {
                    self.head = 0;
                }
            }
        }

        StreamStatus { consumed, produced }
    }

    fn input_needed(&self, output_len: usize) -> Option<usize> {
        // Each output requires `factor` more inputs after the current phase
        // counter; worst case approximation:
        Some(output_len.saturating_mul(self.factor))
    }
}
