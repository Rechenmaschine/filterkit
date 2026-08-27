/// FIR filter coefficients `b[0..N]`.
///
/// `N` is the number of taps. The transfer function is
/// `H(z) = sum_{k=0..N} b[k] * z^(-k)`.
///
/// Pair `FirCoeffs` with [`crate::processors::Fir`] to filter samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FirCoeffs<T, const N: usize> {
    /// Feed-forward taps, indexed from tap 0 (most recent).
    pub b: [T; N],
}

impl<T, const N: usize> FirCoeffs<T, N> {
    /// Wrap a tap array.
    pub const fn new(b: [T; N]) -> Self {
        Self { b }
    }

    /// Number of taps.
    pub const fn len(&self) -> usize {
        N
    }

    /// `true` when there are no taps (only possible for `N = 0`).
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<T: Default + Copy, const N: usize> Default for FirCoeffs<T, N> {
    fn default() -> Self {
        Self {
            b: [T::default(); N],
        }
    }
}

impl<T: Copy, const N: usize> FirCoeffs<T, N> {
    /// Lift this FIR into the equivalent
    /// [`TransferFunction`](crate::coeffs::TransferFunction) with no
    /// denominator taps.
    pub fn to_transfer_function(self) -> crate::coeffs::TransferFunction<T, N, 0> {
        crate::coeffs::TransferFunction::new(self.b, [])
    }
}

impl<T, const N: usize> From<FirCoeffs<T, N>> for crate::coeffs::TransferFunction<T, N, 0> {
    fn from(c: FirCoeffs<T, N>) -> Self {
        Self { b: c.b, a: [] }
    }
}

impl<T, const N: usize> From<crate::coeffs::TransferFunction<T, N, 0>> for FirCoeffs<T, N> {
    fn from(tf: crate::coeffs::TransferFunction<T, N, 0>) -> Self {
        Self { b: tf.b }
    }
}
