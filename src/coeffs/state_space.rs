/// Single-input single-output state-space model of order `N`.
///
/// State recursion:
///
/// ```text
///     x[n+1] = A x[n] + B u[n]
///     y[n]   = C x[n] + D u[n]
/// ```
///
/// Stored as `a: [[T; N]; N]` (row-major), `b: [T; N]`, `c: [T; N]`,
/// `d: T`. Pair with [`crate::processors::StateSpaceProcessor`] to
/// execute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateSpace<T, const N: usize> {
    /// State transition matrix `A` (row-major).
    pub a: [[T; N]; N],
    /// Input matrix `B`.
    pub b: [T; N],
    /// Output matrix `C`.
    pub c: [T; N],
    /// Feed-through scalar `D`.
    pub d: T,
}

impl<T, const N: usize> StateSpace<T, N> {
    /// Wrap the four parameter blocks.
    pub const fn new(a: [[T; N]; N], b: [T; N], c: [T; N], d: T) -> Self {
        Self { a, b, c, d }
    }
}
