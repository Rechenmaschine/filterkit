use crate::traits::{Reset, SampleProcessor};

/// Wraps a processor with a runtime enable flag.
///
/// When `enabled` is `false`, [`process_sample`] returns the input
/// unchanged and the wrapped processor is *not* stepped (its state is
/// frozen). When you flip back to `enabled = true` you may want to
/// [`Reset::reset`] the inner processor first to avoid a click.
///
/// [`process_sample`]: SampleProcessor::process_sample
#[derive(Clone, Copy, Debug)]
pub struct Bypass<F> {
    /// Inner processor.
    pub inner: F,
    /// Whether the processor is active.
    pub enabled: bool,
}

impl<F> Bypass<F> {
    /// Build, enabled by default.
    pub const fn new(inner: F) -> Self {
        Self {
            inner,
            enabled: true,
        }
    }

    /// Toggle.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl<F: Reset> Reset for Bypass<F> {
    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<T, F> SampleProcessor<T> for Bypass<F>
where
    T: Copy,
    F: SampleProcessor<T, Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        if self.enabled {
            self.inner.process_sample(input)
        } else {
            input
        }
    }
}
