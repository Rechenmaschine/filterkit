use super::Reset;

/// Same-rate, block-native processing.
///
/// `BlockProcessor` handles block-based algorithms such as FFT
/// convolution, overlap-add, and SIMD batches.
///
/// Output length matches input length.
///
/// [`SampleProcessor`]: super::SampleProcessor
pub trait BlockProcessor<I>: Reset {
    /// Sample produced for each input sample within the block.
    type Output;

    /// Process one block of input into one block of output. Both slices
    /// must be the same length.
    ///
    /// # Panics
    ///
    /// Implementations should panic if `input.len() != output.len()`.
    fn process_block(&mut self, input: &[I], output: &mut [Self::Output]);
}
