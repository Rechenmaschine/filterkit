//! Exponential moving average design.
//!
//! All three constructors produce a single response-domain `α` in
//! `(0, 1]` suitable for [`crate::processors::Ema`].

/// Spec for an exponential moving average.
///
/// Construct via [`Self::from_alpha`], [`Self::from_time_constant`], or
/// [`Self::from_cutoff_hz`]; then call [`Self::design`] to produce the
/// concrete coefficient.
///
/// All three constructors yield a *response-domain* α: the weight on
/// the new input sample, in `(0, 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExponentialAverageSpec {
    alpha: f64,
}

/// Error type for [`ExponentialAverageSpec`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExponentialAverageError {
    /// `α` was not in `(0, 1]` (after derivation from time-constant /
    /// cutoff if applicable).
    InvalidAlpha,
    /// A spec input that should be strictly positive was not (negative
    /// time constant, zero sample rate, etc.).
    InvalidParameter,
}

impl ExponentialAverageSpec {
    /// Direct construction from `α ∈ (0, 1]`.
    pub fn from_alpha(alpha: f64) -> Result<Self, ExponentialAverageError> {
        if alpha > 0.0 && alpha <= 1.0 {
            Ok(Self { alpha })
        } else {
            Err(ExponentialAverageError::InvalidAlpha)
        }
    }

    /// From a time constant `τ` (seconds) at the given `sample_rate`
    /// (Hz).
    ///
    /// `α = 1 - e^(-T / τ)` where `T = 1 / sample_rate`. Smaller `τ`
    /// ⇒ faster filter ⇒ larger `α`.
    pub fn from_time_constant(
        tau_seconds: f64,
        sample_rate: f64,
    ) -> Result<Self, ExponentialAverageError> {
        if tau_seconds <= 0.0 || sample_rate <= 0.0 {
            return Err(ExponentialAverageError::InvalidParameter);
        }
        let alpha = 1.0 - libm::exp(-1.0 / (tau_seconds * sample_rate));
        Self::from_alpha(alpha)
    }

    /// From a `-3 dB` cutoff frequency `f_c` (Hz) at the given sample
    /// rate.
    ///
    /// Uses the impulse-invariant mapping
    /// `α = 1 - e^(-2π f_c / fs)`. This matches what most audio code
    /// expects when you say "EMA lowpass at X Hz".
    pub fn from_cutoff_hz(
        cutoff_hz: f64,
        sample_rate: f64,
    ) -> Result<Self, ExponentialAverageError> {
        if cutoff_hz <= 0.0 || sample_rate <= 0.0 {
            return Err(ExponentialAverageError::InvalidParameter);
        }
        if cutoff_hz >= sample_rate / 2.0 {
            return Err(ExponentialAverageError::InvalidParameter);
        }
        let alpha = 1.0 - libm::exp(-2.0 * core::f64::consts::PI * cutoff_hz / sample_rate);
        Self::from_alpha(alpha)
    }

    /// Resolved `α` value.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    /// Materialise to an `α` of the target numeric type.
    ///
    /// This is the **coefficient-only** path — useful if you want to
    /// share the same α across several filters or store it for later.
    /// If you just want a ready-to-run filter, prefer [`Self::build`].
    ///
    /// Returns [`ExponentialAverageError::InvalidParameter`] if `T`
    /// cannot represent the underlying `f64` (e.g. NaN, overflow on
    /// fixed-point types).
    pub fn design<T>(&self) -> Result<T, ExponentialAverageError>
    where
        T: num_traits::FromPrimitive,
    {
        T::from_f64(self.alpha).ok_or(ExponentialAverageError::InvalidParameter)
    }

    /// One-step path: materialise this spec straight into a
    /// [`Ema`] processor with zero initial state.
    ///
    /// Equivalent to `Ema::new(spec.design()?)`, just shorter.
    ///
    /// [`Ema`]: crate::processors::Ema
    pub fn build<T>(&self) -> Result<crate::processors::Ema<T>, ExponentialAverageError>
    where
        T: Copy + num_traits::Zero + num_traits::FromPrimitive,
    {
        Ok(crate::processors::Ema::new(self.design::<T>()?))
    }
}
