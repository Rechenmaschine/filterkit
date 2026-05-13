use super::BiquadCoeffs;

/// A fixed-size cascade of second-order sections.
///
/// High-order IIR filters are typically realised as a cascade of biquads
/// to keep numerical conditioning reasonable. The order of `sections`
/// matters: signal flows from `sections[0]` through `sections[N - 1]`.
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

    /// Build an SOS cascade from an iterable of [`BiquadCoeffs`].
    ///
    /// Convenience alias for [`SosCoeffs::new`] taking a const-size
    /// array. The two are interchangeable; this name reads more
    /// naturally in code that originated as a Vec or iterator chain
    /// later collected into an array.
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
