use crate::coeffs::StateSpace;
use crate::traits::{Reset, Retune, SampleProcessor};

/// SISO state-space processor of order `N`.
///
/// Maintains the state vector internally and uses the matrices from
/// [`StateSpace`] for the recurrence
/// `x[n+1] = A x[n] + B u[n]`, `y[n] = C x[n] + D u[n]`.
#[derive(Clone, Copy, Debug)]
pub struct StateSpaceProcessor<T, const N: usize> {
    /// Active model.
    pub coeffs: StateSpace<T, N>,
    state: [T; N],
    /// Scratch buffer for the next-state vector, kept in the struct to
    /// avoid per-sample stack allocation.
    scratch: [T; N],
}

impl<T: Default + Copy, const N: usize> StateSpaceProcessor<T, N> {
    /// Build with zeroed state.
    pub fn new(coeffs: StateSpace<T, N>) -> Self {
        Self {
            coeffs,
            state: [T::default(); N],
            scratch: [T::default(); N],
        }
    }
}

impl<T: Default + Copy, const N: usize> Reset for StateSpaceProcessor<T, N> {
    fn reset(&mut self) {
        self.state = [T::default(); N];
        self.scratch = [T::default(); N];
    }
}

impl<T, const N: usize> Retune<StateSpace<T, N>> for StateSpaceProcessor<T, N> {
    fn retune(&mut self, coeffs: StateSpace<T, N>) {
        self.coeffs = coeffs;
    }
}

impl<T, const N: usize> SampleProcessor<T> for StateSpaceProcessor<T, N>
where
    T: Copy + Default + num_traits::Zero + core::ops::Mul<Output = T> + core::ops::Add<Output = T>,
{
    type Output = T;

    fn process_sample(&mut self, input: T) -> Self::Output {
        let mut y = self.coeffs.d * input;
        for i in 0..N {
            y = y + self.coeffs.c[i] * self.state[i];
        }

        for i in 0..N {
            let mut acc = self.coeffs.b[i] * input;
            for j in 0..N {
                acc = acc + self.coeffs.a[i][j] * self.state[j];
            }
            self.scratch[i] = acc;
        }
        core::mem::swap(&mut self.state, &mut self.scratch);

        y
    }
}
