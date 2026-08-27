//! Frequency response evaluation for filter coefficients.
//!
//! Responses are evaluated at normalised frequencies in `[0, 0.5]` and
//! returned as `f64` values for analysis.

use crate::coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs, TransferFunction};

/// Evaluate `(re, im) = sum_k c[k] * e^{-j 2π f k}` for a real
/// coefficient sequence.
fn eval_z_polynomial(coeffs: impl Iterator<Item = f64>, f: f64) -> (f64, f64) {
    let two_pi_f = 2.0 * core::f64::consts::PI * f;
    let mut re = 0.0;
    let mut im = 0.0;
    for (k, c) in coeffs.enumerate() {
        let phase = two_pi_f * k as f64;
        re += c * libm::cos(phase);
        im -= c * libm::sin(phase);
    }
    (re, im)
}

impl<T, const NB: usize, const NA: usize> TransferFunction<T, NB, NA>
where
    T: Copy + Into<f64>,
{
    /// `|H(e^{j 2π f})|` at the given normalised frequency `f ∈ [0, 0.5]`.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let (nr, ni) = eval_z_polynomial(self.b.iter().map(|&x| x.into()), f);
        let (dr, di) = eval_z_polynomial(
            core::iter::once(1.0).chain(self.a.iter().map(|&x| x.into())),
            f,
        );
        let num = libm::sqrt(nr * nr + ni * ni);
        let den = libm::sqrt(dr * dr + di * di);
        num / den
    }

    /// `arg(H(e^{j 2π f}))` in radians.
    pub fn phase_at(&self, f: f64) -> f64 {
        let (nr, ni) = eval_z_polynomial(self.b.iter().map(|&x| x.into()), f);
        let (dr, di) = eval_z_polynomial(
            core::iter::once(1.0).chain(self.a.iter().map(|&x| x.into())),
            f,
        );
        libm::atan2(ni, nr) - libm::atan2(di, dr)
    }
}

impl<T> BiquadCoeffs<T>
where
    T: Copy + Into<f64>,
{
    /// `|H(e^{j 2π f})|`. Delegates to [`TransferFunction::magnitude_at`].
    pub fn magnitude_at(&self, f: f64) -> f64 {
        self.to_transfer_function().magnitude_at(f)
    }

    /// `arg(H(e^{j 2π f}))` in radians. Delegates to
    /// [`TransferFunction::phase_at`].
    pub fn phase_at(&self, f: f64) -> f64 {
        self.to_transfer_function().phase_at(f)
    }
}

impl<T, const N: usize> FirCoeffs<T, N>
where
    T: Copy + Into<f64>,
{
    /// `|H(e^{j 2π f})|`. Delegates to [`TransferFunction::magnitude_at`].
    pub fn magnitude_at(&self, f: f64) -> f64 {
        self.to_transfer_function().magnitude_at(f)
    }

    /// `arg(H(e^{j 2π f}))` in radians.
    pub fn phase_at(&self, f: f64) -> f64 {
        self.to_transfer_function().phase_at(f)
    }
}

impl<T, const N: usize> SosCoeffs<T, N>
where
    T: Copy + Into<f64>,
{
    /// Magnitude response of the full cascade.
    ///
    /// Cascade magnitudes multiply. Evaluated per section rather than by
    /// flattening to a single transfer function, so the conditioning of
    /// the sectioned form is preserved.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        self.sections.iter().map(|s| s.magnitude_at(f)).product()
    }

    /// Phase response of the full cascade.
    ///
    /// Cascade phases add.
    pub fn phase_at(&self, f: f64) -> f64 {
        self.sections.iter().map(|s| s.phase_at(f)).sum()
    }
}
