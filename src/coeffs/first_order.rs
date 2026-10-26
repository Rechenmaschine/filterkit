/// Coefficients for a general first-order IIR.
///
/// The implemented difference equation is
///
/// ```text
///     y[n] = b0*x[n] + b1*x[n-1] - a1*y[n-1]
/// ```
///
/// The denominator's leading `a0` is implicit and equal to one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FirstOrderCoeffs<T> {
    /// Current-input feed-forward coefficient.
    pub b0: T,
    /// Previous-input feed-forward coefficient.
    pub b1: T,
    /// Previous-output feedback coefficient, with the sign convention
    /// shown in the difference equation above.
    pub a1: T,
}

impl<T> FirstOrderCoeffs<T> {
    /// Construct from normalised coefficients.
    pub const fn new(b0: T, b1: T, a1: T) -> Self {
        Self { b0, b1, a1 }
    }
}

impl<T> FirstOrderCoeffs<T>
where
    T: Copy + core::ops::Div<Output = T>,
{
    /// Construct from an unnormalised four-tuple `(b0, b1, a0, a1)`.
    pub fn from_unnormalised(b0: T, b1: T, a0: T, a1: T) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            a1: a1 / a0,
        }
    }
}

impl<T> FirstOrderCoeffs<T>
where
    T: num_traits::Zero + num_traits::One,
{
    /// Identity transfer function.
    pub fn identity() -> Self {
        Self {
            b0: T::one(),
            b1: T::zero(),
            a1: T::zero(),
        }
    }
}

impl<T: Copy> FirstOrderCoeffs<T> {
    /// Equivalent [`TransferFunction`](crate::coeffs::TransferFunction).
    pub fn to_transfer_function(self) -> crate::coeffs::TransferFunction<T, 2, 1> {
        crate::coeffs::TransferFunction::new([self.b0, self.b1], [self.a1])
    }
}

impl<T: Copy> From<FirstOrderCoeffs<T>> for crate::coeffs::TransferFunction<T, 2, 1> {
    fn from(c: FirstOrderCoeffs<T>) -> Self {
        c.to_transfer_function()
    }
}

impl<T: Copy> From<crate::coeffs::TransferFunction<T, 2, 1>> for FirstOrderCoeffs<T> {
    fn from(tf: crate::coeffs::TransferFunction<T, 2, 1>) -> Self {
        Self::new(tf.b[0], tf.b[1], tf.a[0])
    }
}
