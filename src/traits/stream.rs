use super::Reset;

/// Report of how a [`StreamProcessor`] consumed input and emitted output.
///
/// Variable-rate operators (resamplers, decimators, framers) can't
/// promise output length equals input length, so they hand back explicit
/// counts á la GNU Radio's work functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StreamStatus {
    /// Number of input samples actually read from the input slice.
    pub consumed: usize,
    /// Number of output samples actually written to the output slice.
    pub produced: usize,
}

impl StreamStatus {
    /// Construct from raw counts.
    pub const fn new(consumed: usize, produced: usize) -> Self {
        Self { consumed, produced }
    }
}

/// Variable-rate streaming processor.
///
/// Used for operators where input and output rates differ:
/// decimators, interpolators, polyphase resamplers, arbitrary
/// sample-rate converters, framer/deframers, codec-like adapters.
///
/// `process_stream` reads up to `input.len()` samples and writes up to
/// `output.len()` samples, returning the counts in a [`StreamStatus`].
/// Internal buffering may carry partial frames between calls.
pub trait StreamProcessor<I>: Reset {
    /// Output sample type.
    type Output;

    /// Pull from `input`, push into `output`, return how much of each
    /// was used. Implementations must not panic on size mismatches —
    /// any slice lengths are valid (including empty ones, which is a
    /// drain operation).
    fn process_stream(&mut self, input: &[I], output: &mut [Self::Output]) -> StreamStatus;

    /// Hint for scheduling: how many input samples are needed to
    /// guarantee `output_len` outputs. `None` means the answer is
    /// data-dependent or simply unknown.
    fn input_needed(&self, output_len: usize) -> Option<usize> {
        let _ = output_len;
        None
    }
}
