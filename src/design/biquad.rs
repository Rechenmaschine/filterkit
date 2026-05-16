//! RBJ "audio EQ cookbook" biquad designers.
//!
//! All take a normalised cutoff `f0/fs` (so sample rate cancels out) and
//! return a [`BiquadCoeffs`] in normalised form. `q` is the standard
//! "Q-factor" parameter.
//!
//! Each spec exposes two materialisers:
//! - `design()` — returns [`BiquadCoeffs<f64>`] for reuse / inspection.
//! - `build::<T>()` — returns a ready-to-run [`Biquad<T>`] processor
//!   with zero initial state. Equivalent to
//!   `Biquad::new(spec.design()? converted to T)`.

use crate::coeffs::BiquadCoeffs;
use crate::processors::Biquad;

/// Lowpass biquad spec.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadLowpassSpec {
    /// Normalised cutoff `f0 / fs`.
    pub f0: f64,
    /// Q factor.
    pub q: f64,
}

/// Highpass biquad spec.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadHighpassSpec {
    /// Normalised cutoff `f0 / fs`.
    pub f0: f64,
    /// Q factor.
    pub q: f64,
}

/// Constant-skirt-gain bandpass biquad spec.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadBandpassSpec {
    /// Normalised centre `f0 / fs`.
    pub f0: f64,
    /// Q factor.
    pub q: f64,
}

/// Notch biquad spec.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadNotchSpec {
    /// Normalised centre `f0 / fs`.
    pub f0: f64,
    /// Q factor.
    pub q: f64,
}

/// Error type for biquad design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiquadDesignError {
    /// Normalised frequency outside `(0, 0.5)`.
    InvalidFrequency,
    /// Q was non-positive.
    InvalidQ,
}

fn precompute(f0: f64, q: f64) -> Result<(f64, f64, f64), BiquadDesignError> {
    if !(f0 > 0.0 && f0 < 0.5) {
        return Err(BiquadDesignError::InvalidFrequency);
    }
    if q <= 0.0 {
        return Err(BiquadDesignError::InvalidQ);
    }
    let w0 = 2.0 * core::f64::consts::PI * f0;
    let sn = libm::sin(w0);
    let cs = libm::cos(w0);
    let alpha = sn / (2.0 * q);
    Ok((cs, alpha, sn))
}

impl BiquadLowpassSpec {
    /// Run the design.
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha, _) = precompute(self.f0, self.q)?;
        let b0 = (1.0 - cs) / 2.0;
        let b1 = 1.0 - cs;
        let b2 = (1.0 - cs) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Ok(BiquadCoeffs::from_unnormalised(b0, b1, b2, a0, a1, a2))
    }
}

impl BiquadHighpassSpec {
    /// Run the design.
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha, _) = precompute(self.f0, self.q)?;
        let b0 = (1.0 + cs) / 2.0;
        let b1 = -(1.0 + cs);
        let b2 = (1.0 + cs) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Ok(BiquadCoeffs::from_unnormalised(b0, b1, b2, a0, a1, a2))
    }
}

impl BiquadBandpassSpec {
    /// Run the design (constant skirt gain, peak = Q).
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha, sn) = precompute(self.f0, self.q)?;
        let _ = sn;
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Ok(BiquadCoeffs::from_unnormalised(b0, b1, b2, a0, a1, a2))
    }
}

impl BiquadNotchSpec {
    /// Run the design.
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha, _) = precompute(self.f0, self.q)?;
        let b0 = 1.0;
        let b1 = -2.0 * cs;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Ok(BiquadCoeffs::from_unnormalised(b0, b1, b2, a0, a1, a2))
    }
}

/// Helper: convert an `f64` biquad coefficient set to a generic `T`.
fn coeffs_to<T>(c: BiquadCoeffs<f64>) -> Result<BiquadCoeffs<T>, BiquadDesignError>
where
    T: num_traits::FromPrimitive,
{
    Ok(BiquadCoeffs::new(
        T::from_f64(c.b0).ok_or(BiquadDesignError::InvalidFrequency)?,
        T::from_f64(c.b1).ok_or(BiquadDesignError::InvalidFrequency)?,
        T::from_f64(c.b2).ok_or(BiquadDesignError::InvalidFrequency)?,
        T::from_f64(c.a1).ok_or(BiquadDesignError::InvalidFrequency)?,
        T::from_f64(c.a2).ok_or(BiquadDesignError::InvalidFrequency)?,
    ))
}

/// Numeric bounds for materialising a biquad against a generic `T`.
///
/// Blanket-implemented for any type that is `Copy`, has additive and
/// multiplicative ring structure, a zero, and supports
/// [`num_traits::FromPrimitive`] (needed to cast the `f64` design
/// output into `T`).
pub trait BiquadScalar:
    Copy
    + num_traits::Zero
    + num_traits::FromPrimitive
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + core::ops::Mul<Output = Self>
{
}
impl<T> BiquadScalar for T where
    T: Copy
        + num_traits::Zero
        + num_traits::FromPrimitive
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Mul<Output = T>
{
}

impl BiquadLowpassSpec {
    /// One-step path: design and wrap in a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadHighpassSpec {
    /// One-step path: design and wrap in a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadBandpassSpec {
    /// One-step path: design and wrap in a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadNotchSpec {
    /// One-step path: design and wrap in a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}
