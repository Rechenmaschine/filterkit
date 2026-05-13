use crate::coeffs::FirCoeffs;

/// Specification for an `N`-tap rectangular moving average FIR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovingAverageSpec;

/// Error type for the moving-average designer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovingAverageError {
    /// `N == 0` is not a valid moving average.
    EmptyFilter,
    /// `N` could not be converted to the target numeric type.
    NumericConversion,
}

impl MovingAverageSpec {
    /// Design an `N`-tap moving average FIR with uniform weights `1/N`.
    ///
    /// `N` is supplied via turbofish (`spec.design::<f32, 16>()`); the
    /// compile-time size keeps the result no-alloc and `Copy`.
    pub fn design<T, const N: usize>(self) -> Result<FirCoeffs<T, N>, MovingAverageError>
    where
        T: Copy + num_traits::One + num_traits::FromPrimitive + core::ops::Div<Output = T>,
    {
        if N == 0 {
            return Err(MovingAverageError::EmptyFilter);
        }
        let n_t = T::from_usize(N).ok_or(MovingAverageError::NumericConversion)?;
        let coeff = T::one() / n_t;
        Ok(FirCoeffs::new([coeff; N]))
    }
}
