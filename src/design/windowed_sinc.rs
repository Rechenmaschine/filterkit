use crate::coeffs::FirCoeffs;

/// Tapering windows for windowed-sinc FIR design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Window {
    /// No window (truncated sinc).
    Rectangular,
    /// Hann window. Good general-purpose default for audio.
    Hann,
    /// Hamming window. Slightly better mainlobe than Hann.
    Hamming,
    /// Blackman window. Stronger stop-band rejection at the cost of
    /// wider transition.
    Blackman,
}

/// Spec for a windowed-sinc lowpass FIR with `N` taps.
///
/// The cutoff is given as a *normalised* frequency in `(0, 0.5)` where
/// `0.5` corresponds to the Nyquist limit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowedSincLowpassSpec {
    /// Normalised cutoff frequency, `0 < f < 0.5`.
    pub cutoff: f64,
    /// Window function.
    pub window: Window,
}

/// Error type for windowed-sinc design.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowedSincError {
    /// `cutoff` was outside `(0, 0.5)`.
    InvalidCutoff,
    /// `N` was zero.
    InvalidLength,
}

impl WindowedSincLowpassSpec {
    /// One-step path: design and wrap in a [`Fir`](crate::processors::Fir).
    pub fn build<const N: usize>(
        &self,
    ) -> Result<crate::processors::Fir<f64, N>, WindowedSincError> {
        Ok(crate::processors::Fir::new(self.design::<N>()?))
    }

    /// Design an `N`-tap lowpass FIR with the configured cutoff and
    /// window. Returns coefficients normalised so DC gain is 1.
    pub fn design<const N: usize>(&self) -> Result<FirCoeffs<f64, N>, WindowedSincError> {
        if N == 0 {
            return Err(WindowedSincError::InvalidLength);
        }
        if !(self.cutoff > 0.0 && self.cutoff < 0.5) {
            return Err(WindowedSincError::InvalidCutoff);
        }

        let two_fc = 2.0 * self.cutoff;
        let n_f = N as f64;
        let mid = (n_f - 1.0) / 2.0;

        let mut taps = [0.0f64; N];
        for k in 0..N {
            let nk = k as f64 - mid;
            let s = if nk.abs() < f64::EPSILON {
                // sinc(0) = 1, scaled by 2 fc
                two_fc
            } else {
                let arg = core::f64::consts::PI * two_fc * nk;
                two_fc * libm::sin(arg) / arg
            };
            let w = window_value(self.window, k, N);
            taps[k] = s * w;
        }

        // Normalise DC gain to 1.
        let sum: f64 = taps.iter().sum();
        if sum != 0.0 {
            for t in taps.iter_mut() {
                *t /= sum;
            }
        }

        Ok(FirCoeffs::new(taps))
    }
}

fn window_value(window: Window, k: usize, n: usize) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    let n_f = (n - 1) as f64;
    let k_f = k as f64;
    let two_pi_kn = 2.0 * core::f64::consts::PI * k_f / n_f;

    match window {
        Window::Rectangular => 1.0,
        Window::Hann => 0.5 - 0.5 * libm::cos(two_pi_kn),
        Window::Hamming => 0.54 - 0.46 * libm::cos(two_pi_kn),
        Window::Blackman => 0.42 - 0.5 * libm::cos(two_pi_kn) + 0.08 * libm::cos(2.0 * two_pi_kn),
    }
}
