use nalgebra::{RealField, SMatrix, SVector};

use crate::traits::{Reset, Retune, SampleProcessor};

/// Linear-Gaussian model for an `N`-state, `M`-measurement Kalman filter.
///
/// The model describes the recursion
///
/// ```text
///     x[k] = F x[k-1] + w,   w ~ N(0, Q)      (process)
///     z[k] = H x[k]   + v,   v ~ N(0, R)      (measurement)
/// ```
///
/// The model stores parameters, supports [`Retune`], and is used by
/// [`KalmanFilter`] for estimation.
///
/// The equations follow [Kalman's original formulation](https://doi.org/10.1115/1.3662552).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KalmanModel<T, const N: usize, const M: usize>
where
    T: RealField + Copy,
{
    /// State-transition matrix `F` (`N×N`).
    pub f: SMatrix<T, N, N>,
    /// Measurement matrix `H` (`M×N`).
    pub h: SMatrix<T, M, N>,
    /// Process-noise covariance `Q` (`N×N`).
    pub q: SMatrix<T, N, N>,
    /// Measurement-noise covariance `R` (`M×M`).
    pub r: SMatrix<T, M, M>,
}

impl<T, const N: usize, const M: usize> KalmanModel<T, N, M>
where
    T: RealField + Copy,
{
    /// Assemble a model from its four parameter blocks.
    pub fn new(
        f: SMatrix<T, N, N>,
        h: SMatrix<T, M, N>,
        q: SMatrix<T, N, N>,
        r: SMatrix<T, M, M>,
    ) -> Self {
        Self { f, h, q, r }
    }
}

/// A Gaussian state estimate: a mean vector and its covariance.
///
/// Output of a Kalman update containing both the state estimate and its
/// covariance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GaussianEstimate<T, const N: usize>
where
    T: RealField + Copy,
{
    /// Posterior mean (the point estimate of the state).
    pub mean: SVector<T, N>,
    /// Posterior covariance.
    pub cov: SMatrix<T, N, N>,
}

/// Linear Kalman filter over an `N`-dimensional state estimated from
/// `M`-dimensional measurements.
///
/// The filter maintains a [`GaussianEstimate`] and advances it with two
/// steps:
///
/// - [`predict`](Self::predict) propagates the estimate one step through
///   the process model (call with no measurement to handle gaps).
/// - [`update`](Self::update) folds in a measurement `z`.
///
/// For the common "exactly one measurement per time step" case, the
/// [`SampleProcessor`] impl does `predict` then `update` and returns the
/// posterior estimate, so the filter slots into [`crate::combinators`]
/// like any other processor.
#[derive(Clone, Copy, Debug)]
pub struct KalmanFilter<T, const N: usize, const M: usize>
where
    T: RealField + Copy,
{
    /// Active model.
    pub model: KalmanModel<T, N, M>,
    mean: SVector<T, N>,
    cov: SMatrix<T, N, N>,
}

impl<T, const N: usize, const M: usize> KalmanFilter<T, N, M>
where
    T: RealField + Copy,
{
    /// Build from a model and an explicit prior `(mean, cov)`.
    pub fn new(model: KalmanModel<T, N, M>, mean: SVector<T, N>, cov: SMatrix<T, N, N>) -> Self {
        Self { model, mean, cov }
    }

    /// Build from a model with a zero-mean prior and the given initial
    /// covariance. A large covariance represents greater initial
    /// uncertainty.
    pub fn with_prior_cov(model: KalmanModel<T, N, M>, cov: SMatrix<T, N, N>) -> Self {
        Self::new(model, SVector::zeros(), cov)
    }

    /// Current posterior estimate.
    pub fn estimate(&self) -> GaussianEstimate<T, N> {
        GaussianEstimate {
            mean: self.mean,
            cov: self.cov,
        }
    }

    /// Time update: propagate the estimate one step through the process
    /// model without incorporating a measurement.
    ///
    /// ```text
    ///     x ← F x
    ///     P ← F P Fᵀ + Q
    /// ```
    pub fn predict(&mut self) {
        let f = &self.model.f;
        self.mean = f * self.mean;
        self.cov = f * self.cov * f.transpose() + self.model.q;
    }

    /// Measurement update: fold a measurement `z` into the current
    /// estimate using the optimal Kalman gain.
    ///
    /// Returns the *innovation* `z − H x` (the part of the measurement
    /// the model did not predict), which is handy for residual
    /// monitoring. If the innovation covariance is singular the estimate
    /// is left unchanged and `None` is returned.
    pub fn update(&mut self, z: SVector<T, M>) -> Option<SVector<T, M>> {
        let h = &self.model.h;
        let innovation = z - h * self.mean;
        // Innovation covariance S = H P Hᵀ + R.
        let s = h * self.cov * h.transpose() + self.model.r;
        let s_inv = s.try_inverse()?;
        // Optimal gain K = P Hᵀ S⁻¹.
        let k = self.cov * h.transpose() * s_inv;

        self.mean += k * innovation;
        // Joseph-stabilised covariance update keeps P symmetric positive
        // definite under finite precision: P ← (I−KH) P (I−KH)ᵀ + K R Kᵀ.
        let i = SMatrix::<T, N, N>::identity();
        let ikh = i - k * h;
        self.cov = ikh * self.cov * ikh.transpose() + k * self.model.r * k.transpose();

        Some(innovation)
    }
}

impl<T, const N: usize, const M: usize> Reset for KalmanFilter<T, N, M>
where
    T: RealField + Copy,
{
    /// Reset the mean to zero. The covariance is left as-is so the filter
    /// keeps its notion of prior uncertainty; rebuild with
    /// [`KalmanFilter::with_prior_cov`] to also reset belief.
    fn reset(&mut self) {
        self.mean = SVector::zeros();
    }
}

impl<T, const N: usize, const M: usize> Retune<KalmanModel<T, N, M>> for KalmanFilter<T, N, M>
where
    T: RealField + Copy,
{
    fn retune(&mut self, model: KalmanModel<T, N, M>) {
        self.model = model;
    }
}

impl<T, const N: usize, const M: usize> SampleProcessor<SVector<T, M>> for KalmanFilter<T, N, M>
where
    T: RealField + Copy,
{
    type Output = GaussianEstimate<T, N>;

    /// One predict + update cycle against the measurement `input`,
    /// returning the posterior estimate.
    fn process_sample(&mut self, input: SVector<T, M>) -> Self::Output {
        self.predict();
        self.update(input);
        self.estimate()
    }
}
