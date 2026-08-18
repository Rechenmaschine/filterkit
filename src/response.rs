//! Vector-producing response helpers for analysis and plotting.
//!
//! The per-frequency primitives live on each coefficient type in
//! `crate::design::freq_response`. This module wraps them with:
//!
//! - a [`FrequencyResponse`] trait so downstream code can be generic
//!   over the various coefficient types,
//! - sweep helpers that evaluate magnitude/phase over a frequency grid,
//! - phase unwrapping and (numerical) group delay,
//! - [`impulse_response`] / [`step_response`] for any [`SampleProcessor`].
//!
//! Requires the `alloc` and `design` features.

use alloc::vec::Vec;

use crate::coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs, TransferFunction};
use crate::traits::{Reset, SampleProcessor};

/// Anything that can be queried for `|H(e^{j 2π f})|` and its phase at a
/// normalised frequency `f ∈ [0, 0.5]`.
///
/// Implemented for [`TransferFunction`], [`FirCoeffs`], [`BiquadCoeffs`]
/// and [`SosCoeffs`] for any numeric type that casts to `f64`. Implement
/// it for custom coefficient types to plug them into the sweep helpers
/// and `filterkit-plot`.
pub trait FrequencyResponse {
    /// `|H(e^{j 2π f})|`.
    fn magnitude_at(&self, f: f64) -> f64;

    /// `arg(H(e^{j 2π f}))` in radians.
    fn phase_at(&self, f: f64) -> f64;
}

impl<T, const NB: usize, const NA: usize> FrequencyResponse for TransferFunction<T, NB, NA>
where
    T: Copy + Into<f64>,
{
    fn magnitude_at(&self, f: f64) -> f64 {
        TransferFunction::<T, NB, NA>::magnitude_at(self, f)
    }
    fn phase_at(&self, f: f64) -> f64 {
        TransferFunction::<T, NB, NA>::phase_at(self, f)
    }
}

impl<T, const N: usize> FrequencyResponse for FirCoeffs<T, N>
where
    T: Copy + Into<f64>,
{
    fn magnitude_at(&self, f: f64) -> f64 {
        FirCoeffs::<T, N>::magnitude_at(self, f)
    }
    fn phase_at(&self, f: f64) -> f64 {
        FirCoeffs::<T, N>::phase_at(self, f)
    }
}

impl<T> FrequencyResponse for BiquadCoeffs<T>
where
    T: Copy + Into<f64>,
{
    fn magnitude_at(&self, f: f64) -> f64 {
        BiquadCoeffs::<T>::magnitude_at(self, f)
    }
    fn phase_at(&self, f: f64) -> f64 {
        BiquadCoeffs::<T>::phase_at(self, f)
    }
}

impl<T, const N: usize> FrequencyResponse for SosCoeffs<T, N>
where
    T: Copy + Into<f64>,
{
    fn magnitude_at(&self, f: f64) -> f64 {
        SosCoeffs::<T, N>::magnitude_at(self, f)
    }
    fn phase_at(&self, f: f64) -> f64 {
        SosCoeffs::<T, N>::phase_at(self, f)
    }
}

// Forward references to the response of an owned value through a borrow.
impl<R: FrequencyResponse + ?Sized> FrequencyResponse for &R {
    fn magnitude_at(&self, f: f64) -> f64 {
        (**self).magnitude_at(f)
    }
    fn phase_at(&self, f: f64) -> f64 {
        (**self).phase_at(f)
    }
}

/// `n` evenly-spaced points from `start` to `end` inclusive (like NumPy
/// `linspace`). Returns an empty vector when `n == 0`, and `[start]`
/// when `n == 1`.
pub fn linspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    match n {
        0 => Vec::new(),
        1 => alloc::vec![start],
        _ => {
            let step = (end - start) / (n - 1) as f64;
            (0..n).map(|i| start + step * i as f64).collect()
        }
    }
}

/// `n` log-spaced points from `start` to `end` inclusive. Both bounds
/// must be strictly positive.
pub fn logspace(start: f64, end: f64, n: usize) -> Vec<f64> {
    assert!(start > 0.0 && end > 0.0, "logspace bounds must be positive");
    let log_start = libm::log10(start);
    let log_end = libm::log10(end);
    linspace(log_start, log_end, n)
        .into_iter()
        .map(|x| libm::pow(10.0, x))
        .collect()
}

/// Magnitudes at each normalised frequency in `freqs`.
pub fn magnitude_sweep<R: FrequencyResponse>(r: &R, freqs: &[f64]) -> Vec<f64> {
    freqs.iter().map(|&f| r.magnitude_at(f)).collect()
}

/// Magnitudes in dB (`20 log10 |H|`) at each frequency.
pub fn magnitude_db_sweep<R: FrequencyResponse>(r: &R, freqs: &[f64]) -> Vec<f64> {
    freqs
        .iter()
        .map(|&f| 20.0 * libm::log10(r.magnitude_at(f)))
        .collect()
}

/// Wrapped phases (radians, each in `[-π, π]`) at each frequency.
pub fn phase_sweep<R: FrequencyResponse>(r: &R, freqs: &[f64]) -> Vec<f64> {
    freqs.iter().map(|&f| r.phase_at(f)).collect()
}

/// Unwrapped phases (radians) at each frequency. Equivalent to
/// `phase_sweep` followed by [`unwrap_phase`].
pub fn phase_unwrapped_sweep<R: FrequencyResponse>(r: &R, freqs: &[f64]) -> Vec<f64> {
    let mut p = phase_sweep(r, freqs);
    unwrap_phase(&mut p);
    p
}

/// Convert wrapped phases (each in roughly `[-π, π]`) into a continuous
/// curve by adding ±2π wherever consecutive samples jump by more than
/// π. Operates in place.
pub fn unwrap_phase(phases: &mut [f64]) {
    use core::f64::consts::PI;
    let mut offset = 0.0;
    for i in 1..phases.len() {
        let diff = phases[i] + offset - phases[i - 1];
        if diff > PI {
            offset -= 2.0 * PI;
        } else if diff < -PI {
            offset += 2.0 * PI;
        }
        phases[i] += offset;
    }
}

/// Group delay (in samples) computed numerically from the unwrapped
/// phase via central differences. `freqs` must be sorted ascending.
///
/// `group_delay = -dφ / dω`, with `ω = 2π f`.
///
/// Returns one value per input frequency. End points use one-sided
/// differences.
pub fn group_delay<R: FrequencyResponse>(r: &R, freqs: &[f64]) -> Vec<f64> {
    use core::f64::consts::PI;
    let phases = phase_unwrapped_sweep(r, freqs);
    let n = freqs.len();
    let mut out = alloc::vec![0.0; n];
    if n < 2 {
        return out;
    }
    for i in 0..n {
        let (lo, hi) = if i == 0 {
            (0, 1)
        } else if i == n - 1 {
            (n - 2, n - 1)
        } else {
            (i - 1, i + 1)
        };
        let dphi = phases[hi] - phases[lo];
        let domega = 2.0 * PI * (freqs[hi] - freqs[lo]);
        out[i] = -dphi / domega;
    }
    out
}

/// Run an impulse `[1, 0, 0, ...]` of length `n` through `p` and return
/// the output samples.
///
/// `p` is reset first so the result is independent of prior state.
pub fn impulse_response<P>(p: &mut P, n: usize) -> Vec<f64>
where
    P: SampleProcessor<f64, Output = f64> + Reset,
{
    p.reset();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let x = if i == 0 { 1.0 } else { 0.0 };
        out.push(p.process_sample(x));
    }
    out
}

/// Run a unit step `[1, 1, 1, ...]` of length `n` through `p` and
/// return the output samples.
///
/// `p` is reset first so the result is independent of prior state.
pub fn step_response<P>(p: &mut P, n: usize) -> Vec<f64>
where
    P: SampleProcessor<f64, Output = f64> + Reset,
{
    p.reset();
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(p.process_sample(1.0));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coeffs::FirCoeffs;

    #[test]
    fn linspace_endpoints() {
        let xs = linspace(0.0, 1.0, 5);
        assert_eq!(xs, [0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn logspace_endpoints() {
        let xs = logspace(1.0, 1000.0, 4);
        assert!((xs[0] - 1.0).abs() < 1e-12);
        assert!((xs[3] - 1000.0).abs() < 1e-9);
        // geometric mean property
        assert!((xs[1] - 10.0).abs() < 1e-9);
        assert!((xs[2] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn unwrap_phase_handles_pi_jump() {
        use core::f64::consts::PI;
        // synthetic phase that jumps from +π/2 down past -π
        let mut p = alloc::vec![PI / 2.0, PI - 0.1, -PI + 0.1, -PI / 2.0];
        unwrap_phase(&mut p);
        // No jump greater than π between consecutive samples.
        for w in p.windows(2) {
            assert!((w[1] - w[0]).abs() < PI);
        }
    }

    #[test]
    fn impulse_response_fir_recovers_taps() {
        use crate::processors::Fir;
        let coeffs = FirCoeffs::<f64, 3>::new([0.5, 0.25, 0.125]);
        let mut p: Fir<f64, 3> = Fir::new(coeffs);
        let h = impulse_response(&mut p, 5);
        assert!((h[0] - 0.5).abs() < 1e-12);
        assert!((h[1] - 0.25).abs() < 1e-12);
        assert!((h[2] - 0.125).abs() < 1e-12);
        assert!(h[3].abs() < 1e-12);
        assert!(h[4].abs() < 1e-12);
    }

    #[test]
    fn step_response_fir_sums_taps() {
        use crate::processors::Fir;
        let coeffs = FirCoeffs::<f64, 3>::new([0.5, 0.25, 0.125]);
        let mut p: Fir<f64, 3> = Fir::new(coeffs);
        let s = step_response(&mut p, 5);
        let total: f64 = 0.5 + 0.25 + 0.125;
        assert!((s[4] - total).abs() < 1e-12);
    }
}
