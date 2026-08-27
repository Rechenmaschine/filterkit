use super::BiquadCoeffs;

/// A fixed-size cascade of second-order sections.
///
/// Represents an IIR filter as an ordered cascade of biquads. Signal flows
/// from `sections[0]` through `sections[N - 1]`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SosCoeffs<T, const N: usize> {
    /// Biquad sections, applied in order.
    pub sections: [BiquadCoeffs<T>; N],
}

impl<T, const N: usize> SosCoeffs<T, N> {
    /// Wrap a fixed array of biquad sections.
    pub const fn new(sections: [BiquadCoeffs<T>; N]) -> Self {
        Self { sections }
    }

    /// Build an SOS cascade from an array of [`BiquadCoeffs`].
    pub const fn from_biquads(sections: [BiquadCoeffs<T>; N]) -> Self {
        Self { sections }
    }

    /// Number of sections.
    pub const fn len(&self) -> usize {
        N
    }

    /// `true` when there are no sections (only possible for `N = 0`).
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

impl<T: num_traits::Zero + num_traits::One + Copy, const N: usize> Default for SosCoeffs<T, N> {
    fn default() -> Self {
        Self {
            sections: [BiquadCoeffs::identity(); N],
        }
    }
}
