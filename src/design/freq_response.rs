//! Frequency response evaluation for filter coefficients.
//!
//! `magnitude_at` and `phase_at` compute `|H(e^{j 2π f})|` and
//! `arg(H(...))` for a normalised frequency `f ∈ [0, 0.5]`. All in
//! `f64` for simplicity — for design verification the precision is
//! welcome, and these are not on the hot path.

use crate::coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs};

/// Evaluate `(re, im) = sum_k c[k] * e^{-j 2π f k}` for real-valued
/// coefficient arrays.
fn dft_at(coeffs: impl Iterator<Item = f64>, f: f64) -> (f64, f64) {
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

impl<const N: usize> FirCoeffs<f64, N> {
    /// `|H(e^{j 2π f})|` at the given normalised frequency `f ∈ [0, 0.5]`.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let (re, im) = dft_at(self.b.iter().copied(), f);
        libm::sqrt(re * re + im * im)
    }

    /// `arg(H(e^{j 2π f}))` in radians.
    pub fn phase_at(&self, f: f64) -> f64 {
        let (re, im) = dft_at(self.b.iter().copied(), f);
        libm::atan2(im, re)
    }
}

impl<const N: usize> FirCoeffs<f32, N> {
    /// Magnitude response at normalised frequency `f`. See the `f64`
    /// variant; this delegates after widening.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let (re, im) = dft_at(self.b.iter().map(|&x| x as f64), f);
        libm::sqrt(re * re + im * im)
    }

    /// Phase response in radians.
    pub fn phase_at(&self, f: f64) -> f64 {
        let (re, im) = dft_at(self.b.iter().map(|&x| x as f64), f);
        libm::atan2(im, re)
    }
}

impl BiquadCoeffs<f64> {
    /// `|H(e^{j 2π f})|`.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let (nr, ni) = dft_at([self.b0, self.b1, self.b2].into_iter(), f);
        let (dr, di) = dft_at([1.0, self.a1, self.a2].into_iter(), f);
        let num = libm::sqrt(nr * nr + ni * ni);
        let den = libm::sqrt(dr * dr + di * di);
        num / den
    }

    /// Phase response in radians.
    pub fn phase_at(&self, f: f64) -> f64 {
        let (nr, ni) = dft_at([self.b0, self.b1, self.b2].into_iter(), f);
        let (dr, di) = dft_at([1.0, self.a1, self.a2].into_iter(), f);
        libm::atan2(ni, nr) - libm::atan2(di, dr)
    }
}

impl BiquadCoeffs<f32> {
    /// Magnitude response.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let widened: BiquadCoeffs<f64> = BiquadCoeffs::new(
            self.b0 as f64,
            self.b1 as f64,
            self.b2 as f64,
            self.a1 as f64,
            self.a2 as f64,
        );
        widened.magnitude_at(f)
    }

    /// Phase response.
    pub fn phase_at(&self, f: f64) -> f64 {
        let widened: BiquadCoeffs<f64> = BiquadCoeffs::new(
            self.b0 as f64,
            self.b1 as f64,
            self.b2 as f64,
            self.a1 as f64,
            self.a2 as f64,
        );
        widened.phase_at(f)
    }
}

impl<const N: usize> SosCoeffs<f64, N> {
    /// Magnitude response of the full cascade.
    ///
    /// Cascade magnitudes multiply.
    pub fn magnitude_at(&self, f: f64) -> f64 {
        let mut prod = 1.0;
        for s in self.sections.iter() {
            prod *= s.magnitude_at(f);
        }
        prod
    }

    /// Phase response of the full cascade.
    ///
    /// Cascade phases add.
    pub fn phase_at(&self, f: f64) -> f64 {
        let mut sum = 0.0;
        for s in self.sections.iter() {
            sum += s.phase_at(f);
        }
        sum
    }
}
