/// Parameters provided to a processor before it runs.
///
/// The numeric type `T` is generic so processors can use types other than
/// `f64`.
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

/// Optional setup hook for processors that allocate or validate against
/// host parameters.
pub trait Prepare<T> {
    /// Error returned when the spec is unworkable (e.g. block too large).
    type Error;

    /// Configure the processor against `spec`. After a successful call,
    /// the processor must accept blocks up to `spec.max_block_len` long
    /// without further allocation.
    fn prepare(&mut self, spec: ProcessSpec<T>) -> Result<(), Self::Error>;
}
