use crate::traits::{Reset, Retune, SampleProcessor};

/// One-pole IIR / exponential moving average.
///
/// Difference equation:
///
/// ```text
///     y[n] = α · x[n] + (1 - α) · y[n - 1]
/// ```
///
/// `α ∈ (0, 1]`. Smaller `α` ⇒ slower, smoother response. `α = 1`
/// passes input through unchanged.
///
/// Construct directly with [`OnePole::new`] if you already have an
/// `α`, or use [`crate::design::ExponentialAverageSpec`] when you'd
/// rather think in terms of a time-constant or cutoff frequency.
///
/// # Note on parameterisation
///
/// Some references write `y[n] = (1 - α) · x[n] + α · y[n - 1]` (the
/// "pole-domain" α). This crate uses the *response-domain* α — i.e.
/// `α` is the weight on the *new* input sample. They are simply
/// complements of each other.
#[derive(Clone, Copy, Debug)]
pub struct OnePole<T> {
    /// Mix factor applied to the new input. Bigger α ⇒ less smoothing.
    pub alpha: T,
    y: T,
}

impl<T> OnePole<T>
where
    T: num_traits::Zero + Copy,
{
    /// Build a one-pole filter with zero initial state.
    pub fn new(alpha: T) -> Self {
        Self { alpha, y: T::zero() }
    }

    /// Build with a pre-loaded last-output value, for stitching blocks.
    pub fn with_state(alpha: T, last_output: T) -> Self {
        Self { alpha, y: last_output }
    }

    /// Current output (the `y[n-1]` that the next call will see).
    pub fn last_output(&self) -> T
    where
        T: Copy,
    {
        self.y
    }
}

impl<T> Reset for OnePole<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        self.y = T::zero();
    }
}

impl<T> Retune<T> for OnePole<T> {
    /// Replace `α` without disturbing state. State carryover is the
    /// usual choice for one-poles since changing `α` is a smooth
    /// modulation gesture.
    fn retune(&mut self, alpha: T) {
        self.alpha = alpha;
    }
}

impl<T> SampleProcessor<T> for OnePole<T>
where
    T: Copy
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let one = T::one();
        self.y = self.alpha * input + (one - self.alpha) * self.y;
        self.y
    }
}
