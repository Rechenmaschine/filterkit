//! **Prototype**: high-level filter specs that pick a kernel for you.
//!
//! Where the rest of [`crate::design`] takes a spec and hands back
//! *coefficients* (forcing the user to choose between `OnePole`,
//! `Biquad`, `SosCascade`, …), the specs in this module return a
//! ready-built processor wrapped in a kernel-erased enum like
//! [`Lowpass`].
//!
//! The library is then free to pick the cheapest kernel that satisfies
//! the request — e.g. a 1st-order lowpass becomes [`OnePole`] (one
//! state word, 2 muls), a 2nd-order lowpass becomes [`Biquad`]
//! (DF2T, two state words). The user writes the same code in both
//! cases.
//!
//! ## Trade-off
//!
//! The enum is sized by its largest variant, and `process_sample`
//! dispatches through a `match`. In practice the branch predictor
//! sees the same variant on every call, so the cost is at most a
//! correctly-predicted conditional branch — not zero, but very close.
//! When you need *guaranteed* zero overhead, drop down to the
//! kernel-specific specs ([`super::BiquadLowpassSpec`],
//! [`super::ExponentialAverageSpec`], …) and own the dispatch yourself.
//!
//! ## Status
//!
//! This is a prototype. Only [`Lowpass`] / [`LowpassSpec`] are
//! implemented; higher-order (≥ 3) routes through cascaded biquads
//! are stubbed (see [`LowpassBuildError::UnsupportedOrder`]).

use crate::design::{BiquadDesignError, BiquadLowpassSpec, ExponentialAverageError, ExponentialAverageSpec};
use crate::processors::{Biquad, OnePole};
use crate::traits::{Reset, SampleProcessor};

/// High-level lowpass request.
///
/// Specify cutoff, sample rate, and filter order; the library picks
/// the kernel. `order = 1` runs cheaper than `order = 2`, which runs
/// cheaper than higher-order cascades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LowpassSpec {
    /// `-3 dB` cutoff in Hertz.
    pub cutoff_hz: f64,
    /// Sample rate in Hertz.
    pub sample_rate: f64,
    /// Filter order. `1` ⇒ one-pole; `2` ⇒ biquad.
    pub order: usize,
}

/// Error from [`LowpassSpec::build`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LowpassBuildError {
    /// `order = 0` or higher than the prototype supports.
    UnsupportedOrder { requested: usize },
    /// Upstream EMA designer rejected the parameters.
    ExponentialAverage(ExponentialAverageError),
    /// Upstream biquad designer rejected the parameters.
    Biquad(BiquadDesignError),
    /// The numeric type couldn't represent the designed coefficient
    /// (NaN, overflow on fixed-point, …).
    NumericConversion,
}

impl From<ExponentialAverageError> for LowpassBuildError {
    fn from(e: ExponentialAverageError) -> Self {
        Self::ExponentialAverage(e)
    }
}

impl From<BiquadDesignError> for LowpassBuildError {
    fn from(e: BiquadDesignError) -> Self {
        Self::Biquad(e)
    }
}

/// Kernel-erased lowpass processor.
///
/// Constructed by [`LowpassSpec::build`]. Internally one of:
/// - [`OnePole`] for `order = 1`
/// - [`Biquad`] for `order = 2`
///
/// Implements [`SampleProcessor`] and [`Reset`] uniformly; the `match`
/// dispatch happens inside `process_sample`.
#[derive(Clone, Copy, Debug)]
pub enum Lowpass<T> {
    /// 1st-order kernel.
    OnePole(OnePole<T>),
    /// 2nd-order kernel.
    Biquad(Biquad<T>),
}

impl<T> Reset for Lowpass<T>
where
    T: num_traits::Zero + Copy,
{
    fn reset(&mut self) {
        match self {
            Lowpass::OnePole(p) => p.reset(),
            Lowpass::Biquad(p) => p.reset(),
        }
    }
}

impl<T> SampleProcessor<T> for Lowpass<T>
where
    T: Copy
        + num_traits::Zero
        + num_traits::One
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Mul<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        match self {
            Lowpass::OnePole(p) => p.process_sample(input),
            Lowpass::Biquad(p) => p.process_sample(input),
        }
    }

    /// Overridden to hoist the variant `match` *out* of the per-sample
    /// loop.
    ///
    /// The default impl from [`SampleProcessor`] would call our
    /// `process_sample` once per sample, which matches each time and
    /// blocks LLVM from optimising across iterations. Here we match
    /// once and run the concrete kernel's own `process_into` over the
    /// whole block — getting (in benchmarks) ≈ concrete-kernel speed
    /// without giving up the enum-erased return type from
    /// [`LowpassSpec::build`].
    ///
    /// Same idea as match-on-the-outside-of-the-hot-loop in any
    /// hand-written variable-kernel DSP code.
    fn process_into(&mut self, input: &[T], output: &mut [T]) {
        assert_eq!(
            input.len(),
            output.len(),
            "process_into: input and output must have equal length",
        );
        match self {
            Lowpass::OnePole(p) => p.process_into(input, output),
            Lowpass::Biquad(p) => p.process_into(input, output),
        }
    }
}

impl LowpassSpec {
    /// Build a lowpass with the library's choice of kernel.
    ///
    /// The kernel is selected from the [`order`](LowpassSpec::order)
    /// field. For explicit kernel selection that doesn't go through
    /// the [`Lowpass`] enum, use [`Self::as_one_pole`] or
    /// [`Self::as_biquad`].
    ///
    /// `T` is the runtime sample type; the design math runs in `f64`
    /// then converts via [`num_traits::FromPrimitive`].
    pub fn build<T>(&self) -> Result<Lowpass<T>, LowpassBuildError>
    where
        T: Copy
            + num_traits::Zero
            + num_traits::One
            + num_traits::FromPrimitive
            + core::ops::Add<Output = T>
            + core::ops::Sub<Output = T>
            + core::ops::Mul<Output = T>,
    {
        match self.order {
            0 => Err(LowpassBuildError::UnsupportedOrder { requested: 0 }),
            1 => self.as_one_pole().map(Lowpass::OnePole),
            2 => self.as_biquad().map(Lowpass::Biquad),
            // Higher orders deliberately stubbed in the prototype;
            // reserved for `Lowpass::Sos(SosDyn<T>)` once Butterworth/
            // Chebyshev SOS design lands.
            n => Err(LowpassBuildError::UnsupportedOrder { requested: n }),
        }
    }

    /// Materialise this spec as a [`OnePole`] regardless of the
    /// [`order`](LowpassSpec::order) field.
    ///
    /// Returns a concrete type — no enum dispatch, fully composable
    /// with combinators like [`crate::combinators::Chain`].
    ///
    /// Uses the impulse-invariant mapping
    /// `α = 1 - exp(-2π f_c / fs)`.
    pub fn as_one_pole<T>(&self) -> Result<OnePole<T>, LowpassBuildError>
    where
        T: Copy
            + num_traits::Zero
            + num_traits::FromPrimitive,
    {
        let ema = ExponentialAverageSpec::from_cutoff_hz(self.cutoff_hz, self.sample_rate)?;
        let alpha: T = ema.design().map_err(LowpassBuildError::from)?;
        Ok(OnePole::new(alpha))
    }

    /// Materialise this spec as a [`Biquad`] regardless of the
    /// [`order`](LowpassSpec::order) field.
    ///
    /// Uses the RBJ cookbook formula with Butterworth `Q = 1/√2`.
    pub fn as_biquad<T>(&self) -> Result<Biquad<T>, LowpassBuildError>
    where
        T: Copy
            + num_traits::Zero
            + num_traits::FromPrimitive
            + core::ops::Add<Output = T>
            + core::ops::Sub<Output = T>
            + core::ops::Mul<Output = T>,
    {
        let spec = BiquadLowpassSpec {
            f0: self.cutoff_hz / self.sample_rate,
            q: core::f64::consts::FRAC_1_SQRT_2,
        };
        let c64 = spec.design()?;
        let coeffs = crate::coeffs::BiquadCoeffs::new(
            T::from_f64(c64.b0).ok_or(LowpassBuildError::NumericConversion)?,
            T::from_f64(c64.b1).ok_or(LowpassBuildError::NumericConversion)?,
            T::from_f64(c64.b2).ok_or(LowpassBuildError::NumericConversion)?,
            T::from_f64(c64.a1).ok_or(LowpassBuildError::NumericConversion)?,
            T::from_f64(c64.a2).ok_or(LowpassBuildError::NumericConversion)?,
        );
        Ok(Biquad::new(coeffs))
    }

    /// **Prototype**: same dispatch as [`build`](Self::build), but the
    /// kernel is returned as a heap-allocated trait object.
    ///
    /// Compared to [`build`](Self::build) returning [`Lowpass<T>`]:
    ///
    /// | Trait object | Enum |
    /// |---|---|
    /// | Heap allocation per filter | Stack only |
    /// | Dynamic dispatch (vtable lookup) per `process_sample` | `match` (branch-predicted) |
    /// | Kernel set extensible by downstream crates | Closed (enum variants only) |
    /// | Does **not** compose with [`ProcessorExt`] (`!Sized`) | Composes freely |
    /// | Hides the concrete kernel from callers | Callers can pattern-match |
    ///
    /// In benchmarks the vtable cost is typically 1–3 ns/sample on
    /// modern x86_64 — small for one filter, accumulates for thousands.
    /// Use this when you genuinely need open kernel extensibility
    /// (e.g. user-pluggable filter kernels); prefer [`build`](Self::build)
    /// otherwise.
    ///
    /// [`ProcessorExt`]: crate::traits::ProcessorExt
    #[cfg(feature = "alloc")]
    pub fn build_boxed<T>(
        &self,
    ) -> Result<alloc::boxed::Box<dyn crate::traits::SampleProcessor<T, Output = T>>, LowpassBuildError>
    where
        T: 'static
            + Copy
            + num_traits::Zero
            + num_traits::One
            + num_traits::FromPrimitive
            + core::ops::Add<Output = T>
            + core::ops::Sub<Output = T>
            + core::ops::Mul<Output = T>,
    {
        match self.order {
            0 => Err(LowpassBuildError::UnsupportedOrder { requested: 0 }),
            1 => Ok(alloc::boxed::Box::new(self.as_one_pole::<T>()?)),
            2 => Ok(alloc::boxed::Box::new(self.as_biquad::<T>()?)),
            n => Err(LowpassBuildError::UnsupportedOrder { requested: n }),
        }
    }
}
