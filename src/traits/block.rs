use super::Reset;

/// Same-rate, block-native processing.
///
/// `BlockProcessor` is for algorithms whose natural unit is a block of
/// samples — FFT convolution, overlap-add / overlap-save, partitioned
/// convolution, SIMD-optimised FIRs, and multi-channel block processors.
/// They could be wrapped to look like a [`SampleProcessor`], but doing so
/// often defeats their reason for existing.
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
