/// Rational transfer function `H(z) = B(z) / A(z)` in `z^{-1}` form.
///
/// `b` holds the numerator (feed-forward) taps, `a` the denominator
/// (feed-back) taps. Convention: `a[0]` is the implicit unity and is
/// *not* stored — the first stored coefficient is `a[1]` and corresponds
/// to `y[n-1]` in
///
/// ```text
///     y[n] = sum_k b[k] * x[n-k]  -  sum_{k>=1} a[k] * y[n-k]
/// ```
///
/// `NB` is the number of feed-forward taps; `NA` is the number of
/// feed-back taps stored (so the textbook denominator has `NA + 1`
/// terms).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferFunction<T, const NB: usize, const NA: usize> {
    /// Feed-forward taps `b[0..NB]`.
    pub b: [T; NB],
    /// Feed-back taps `a[1..=NA]` (the `a[0] = 1` term is implicit).
    pub a: [T; NA],
}

impl<T, const NB: usize, const NA: usize> TransferFunction<T, NB, NA> {
    /// Wrap raw numerator/denominator arrays. `a` must *not* include the
    /// leading `a[0] = 1` term.
    pub const fn new(b: [T; NB], a: [T; NA]) -> Self {
        Self { b, a }
    }
}
