/// Second-order section (biquad) coefficients in normalised form.
///
/// The implemented difference equation is
///
/// ```text
///     y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]
///                    - a1*y[n-1] - a2*y[n-2]
/// ```
///
/// i.e. `a0` is taken to be 1; designers are expected to normalise.
/// (`Self::from_unnormalised` does it for you when given raw `a0`.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BiquadCoeffs<T> {
    /// `b[0]` feed-forward.
    pub b0: T,
    /// `b[1]` feed-forward.
    pub b1: T,
    /// `b[2]` feed-forward.
    pub b2: T,
    /// `a[1]` feed-back (sign as in the difference equation above, *not*
    /// negated — i.e. the value you'd read from a textbook transfer
    /// function denominator).
    pub a1: T,
    /// `a[2]` feed-back.
    pub a2: T,
}

impl<T> BiquadCoeffs<T> {
    /// Construct from the five normalised coefficients.
    pub const fn new(b0: T, b1: T, b2: T, a1: T, a2: T) -> Self {
        Self { b0, b1, b2, a1, a2 }
    }
}

impl<T> BiquadCoeffs<T>
where
    T: Copy + core::ops::Div<Output = T>,
{
    /// Construct from an unnormalised six-tuple `(b0, b1, b2, a0, a1, a2)`
    /// by dividing through by `a0`.
    pub fn from_unnormalised(b0: T, b1: T, b2: T, a0: T, a1: T, a2: T) -> Self {
        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

impl<T: num_traits::Zero + num_traits::One> BiquadCoeffs<T> {
    /// Identity section: passes input through unchanged.
    pub fn identity() -> Self {
        Self {
            b0: T::one(),
            b1: T::zero(),
            b2: T::zero(),
            a1: T::zero(),
            a2: T::zero(),
        }
    }
}

impl<T: Copy> BiquadCoeffs<T> {
    /// Lift this biquad into the equivalent
    /// [`TransferFunction`](crate::coeffs::TransferFunction) with three
    /// numerator coefficients and two denominator coefficients.
    pub fn to_transfer_function(self) -> crate::coeffs::TransferFunction<T, 3, 2> {
        crate::coeffs::TransferFunction::new([self.b0, self.b1, self.b2], [self.a1, self.a2])
    }
}

impl<T: Copy> From<BiquadCoeffs<T>> for crate::coeffs::TransferFunction<T, 3, 2> {
    fn from(c: BiquadCoeffs<T>) -> Self {
        c.to_transfer_function()
    }
}
