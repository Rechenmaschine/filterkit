/// Processors that operate on a whole finite signal at once.
///
/// Unlike [`SampleProcessor`] or [`BlockProcessor`], a
/// `WholeSignalProcessor` is allowed to be non-causal — e.g. zero-phase
/// filtering (`filtfilt`), centered moving averages, Savitzky-Golay
/// smoothing, batch spectral processing.
///
/// No `Reset` requirement: whole-signal processors are typically
/// stateless across calls or carry only design-time data.
///
/// [`SampleProcessor`]: super::SampleProcessor
/// [`BlockProcessor`]: super::BlockProcessor
pub trait WholeSignalProcessor<I> {
    /// Sample produced for each input sample.
    type Output;

    /// Process the entire input into the entire output.
    ///
    /// # Panics
    ///
    /// Implementations should panic if `input.len() != output.len()`.
    fn process_whole(&mut self, input: &[I], output: &mut [Self::Output]);
}
