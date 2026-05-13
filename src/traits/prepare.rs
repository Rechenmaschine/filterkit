/// Stream parameters announced to a processor before it runs.
///
/// `ProcessSpec` is single-channel in 0.1 — multi-channel processors are
/// out of scope for this release. The numeric type `T` is parametric so
/// fixed-point or integer-rate processors aren't forced into `f64`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessSpec<T> {
    /// Audio/data sample rate in Hertz.
    pub sample_rate: T,
    /// Maximum number of samples the processor may see in a single call.
    pub max_block_len: usize,
}

impl<T> ProcessSpec<T> {
    /// Convenience constructor.
    pub const fn new(sample_rate: T, max_block_len: usize) -> Self {
        Self {
            sample_rate,
            max_block_len,
        }
    }
}

/// Optional pre-run setup hook for processors that need to allocate or
/// validate against host parameters.
///
/// Pure no_std processors typically do not implement this — they are
/// fully configured by their representations. Allocating processors,
/// resamplers, and FFT-based blocks usually do.
pub trait Prepare<T> {
    /// Error returned when the spec is unworkable (e.g. block too large).
    type Error;

    /// Configure the processor against `spec`. After a successful call,
    /// the processor must accept blocks up to `spec.max_block_len` long
    /// without further allocation.
    fn prepare(&mut self, spec: ProcessSpec<T>) -> Result<(), Self::Error>;
}
