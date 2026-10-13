//! RBJ "audio EQ cookbook" biquad designers.
//!
//! Specs take a normalised cutoff `f0/fs` and return normalised
//! [`BiquadCoeffs`]. Use `design()` for coefficients or `build()` for a
//! ready-to-run processor.
//!
//! The formulas follow the
//! [Audio EQ Cookbook](https://www.w3.org/TR/audio-eq-cookbook/) by Robert
//! Bristow-Johnson.

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

/// Bandpass biquad spec with a constant 0 dB peak gain.
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
    /// Q was non-positive or NaN.
    InvalidQ,
    /// A designed coefficient could not be converted to the target
    /// numeric type.
    NumericConversion,
}

fn precompute(f0: f64, q: f64) -> Result<(f64, f64), BiquadDesignError> {
    if !(f0 > 0.0 && f0 < 0.5) {
        return Err(BiquadDesignError::InvalidFrequency);
    }
    if q.is_nan() || q <= 0.0 {
        return Err(BiquadDesignError::InvalidQ);
    }
    let w0 = 2.0 * core::f64::consts::PI * f0;
    let sn = libm::sin(w0);
    let cs = libm::cos(w0);
    let alpha = sn / (2.0 * q);
    Ok((cs, alpha))
}

impl BiquadLowpassSpec {
    /// Run the design.
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha) = precompute(self.f0, self.q)?;
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
        let (cs, alpha) = precompute(self.f0, self.q)?;
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
    /// Run the design (constant 0 dB peak gain).
    pub fn design(&self) -> Result<BiquadCoeffs<f64>, BiquadDesignError> {
        let (cs, alpha) = precompute(self.f0, self.q)?;
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
        let (cs, alpha) = precompute(self.f0, self.q)?;
        let b0 = 1.0;
        let b1 = -2.0 * cs;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cs;
        let a2 = 1.0 - alpha;
        Ok(BiquadCoeffs::from_unnormalised(b0, b1, b2, a0, a1, a2))
    }
}

fn coeffs_to<T>(c: BiquadCoeffs<f64>) -> Result<BiquadCoeffs<T>, BiquadDesignError>
where
    T: num_traits::FromPrimitive,
{
    Ok(BiquadCoeffs::new(
        T::from_f64(c.b0).ok_or(BiquadDesignError::NumericConversion)?,
        T::from_f64(c.b1).ok_or(BiquadDesignError::NumericConversion)?,
        T::from_f64(c.b2).ok_or(BiquadDesignError::NumericConversion)?,
        T::from_f64(c.a1).ok_or(BiquadDesignError::NumericConversion)?,
        T::from_f64(c.a2).ok_or(BiquadDesignError::NumericConversion)?,
    ))
}

/// Numeric requirements for converting designed coefficients to `T`.
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
    /// Design and build a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadHighpassSpec {
    /// Design and build a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadBandpassSpec {
    /// Design and build a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}

impl BiquadNotchSpec {
    /// Design and build a [`Biquad`].
    pub fn build<T: BiquadScalar>(&self) -> Result<Biquad<T>, BiquadDesignError> {
        Ok(Biquad::new(coeffs_to::<T>(self.design()?)?))
    }
}
