use crate::coeffs::TransferFunction;
use crate::traits::{FiltFiltKernel, Reset, Retune, SampleProcessor, SteadyState};

/// Direct-form I IIR realisation of a [`TransferFunction`].
///
/// Holds `NB` past inputs and `NA` past outputs. Numerically less
/// forgiving than DF2T at high order — for serious IIRs prefer
/// [`crate::processors::SosCascade`] — but useful for low-order designs
/// and as a reference implementation.
///
/// # Design note
///
/// `DirectFormIITransposed` for arbitrary `(NB, NA)` orders is *not*
/// exposed in 0.1: it would require carrying a separate const generic
/// `NS = max(NB, NA + 1) - 1`, which stable Rust can't derive. Second-
/// order DF2T is available via [`crate::processors::Biquad`] and
/// [`crate::processors::SosCascade`].
#[derive(Clone, Copy, Debug)]
pub struct DirectFormI<T, const NB: usize, const NA: usize> {
    /// Active transfer function.
    pub coeffs: TransferFunction<T, NB, NA>,
    x: [T; NB],
    y: [T; NA],
}

impl<T: Default + Copy, const NB: usize, const NA: usize> DirectFormI<T, NB, NA> {
    /// Build with zero past inputs/outputs.
    pub fn new(coeffs: TransferFunction<T, NB, NA>) -> Self {
        Self {
            coeffs,
            x: [T::default(); NB],
            y: [T::default(); NA],
        }
    }
}

impl<T: Default + Copy, const NB: usize, const NA: usize> Reset for DirectFormI<T, NB, NA> {
    fn reset(&mut self) {
        self.x = [T::default(); NB];
        self.y = [T::default(); NA];
    }
}

impl<T, const NB: usize, const NA: usize> Retune<TransferFunction<T, NB, NA>>
    for DirectFormI<T, NB, NA>
{
    fn retune(&mut self, coeffs: TransferFunction<T, NB, NA>) {
        self.coeffs = coeffs;
    }
}

impl<T, const NB: usize, const NA: usize> SampleProcessor<T> for DirectFormI<T, NB, NA>
where
    T: Copy
        + Default
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        // y[n] = sum_k b[k]*x[n-k] - sum_{k>=1} a[k]*y[n-k]
        // x[k] for k >= 1 stored as: self.x[k-1] = x[n-k] from previous step.
        let mut acc = T::zero();

        if NB > 0 {
            acc = acc + self.coeffs.b[0] * input;
            for k in 1..NB {
                acc = acc + self.coeffs.b[k] * self.x[k - 1];
            }
        }
        for k in 0..NA {
            acc = acc - self.coeffs.a[k] * self.y[k];
        }

        if NB > 1 {
            for k in (1..NB - 1).rev() {
                self.x[k] = self.x[k - 1];
            }
            self.x[0] = input;
        }

        if NA > 0 {
            for k in (1..NA).rev() {
                self.y[k] = self.y[k - 1];
            }
            self.y[0] = acc;
        }

        acc
    }
}

impl<T, const NB: usize, const NA: usize> SteadyState<T> for DirectFormI<T, NB, NA>
where
    T: Copy
        + Default
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn reset_to_steady_input(&mut self, input: T) {
        let mut numerator = T::zero();
        for b in self.coeffs.b {
            numerator = numerator + b;
        }

        let mut denominator = T::one();
        for a in self.coeffs.a {
            denominator = denominator + a;
        }

        let steady = numerator * input / denominator;
        self.x = [input; NB];
        self.y = [steady; NA];
    }
}

impl<T, const NB: usize, const NA: usize> FiltFiltKernel<T> for DirectFormI<T, NB, NA>
where
    T: Copy
        + Default
        + num_traits::One
        + num_traits::Zero
        + core::ops::Mul<Output = T>
        + core::ops::Add<Output = T>
        + core::ops::Sub<Output = T>
        + core::ops::Div<Output = T>,
{
    fn filtfilt_pad_len(&self) -> usize {
        3 * NB.max(NA + 1)
    }
}
