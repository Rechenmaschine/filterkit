use alloc::vec::Vec;

use crate::traits::{FiltFiltKernel, WholeSignalProcessor};

/// Edge extension used before the forward/backward passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadType {
    /// Odd reflection around each endpoint. This is SciPy's default.
    Odd,
    /// Even reflection around each endpoint.
    Even,
    /// Repeat the endpoint values.
    Constant,
    /// Do not pad the signal.
    None,
}

/// Zero-phase forward/backward filtering a la SciPy's `filtfilt`.
///
/// Runs the wrapped causal filter forward over an edge-extended signal,
/// then backward over the forward result. Before each pass, the wrapped
/// filter is initialized to its steady state for the pass's first sample,
/// matching SciPy's default `method="pad"` strategy. The net frequency
/// response is the squared magnitude of the underlying filter with zero
/// phase.
///
/// Requires the `alloc` feature.
#[derive(Debug)]
pub struct ForwardBackward<F> {
    /// Wrapped causal filter.
    pub filter: F,
    /// Reflection/extension style. Defaults to [`PadType::Odd`].
    pub pad_type: PadType,
    /// Explicit padding length. `None` uses the filter's SciPy-style
    /// default; the effective value is clamped for very short signals.
    pub pad_len: Option<usize>,
}

impl<F> ForwardBackward<F> {
    /// Wrap a causal filter using odd reflection and an automatic pad
    /// length.
    pub const fn new(filter: F) -> Self {
        Self {
            filter,
            pad_type: PadType::Odd,
            pad_len: None,
        }
    }

    /// Set the edge extension style.
    pub const fn with_pad_type(mut self, pad_type: PadType) -> Self {
        self.pad_type = pad_type;
        self
    }

    /// Set an explicit edge length.
    pub const fn with_pad_len(mut self, pad_len: usize) -> Self {
        self.pad_len = Some(pad_len);
        self
    }

    /// Disable edge padding.
    pub const fn without_padding(mut self) -> Self {
        self.pad_type = PadType::None;
        self.pad_len = Some(0);
        self
    }
}

impl<T, F> WholeSignalProcessor<T> for ForwardBackward<F>
where
    T: Copy + core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
    F: FiltFiltKernel<T>,
{
    type Output = T;

    fn process_whole(&mut self, input: &[T], output: &mut [Self::Output]) {
        assert_eq!(
            input.len(),
            output.len(),
            "process_whole: input and output must have equal length",
        );

        if input.is_empty() {
            return;
        }

        let edge = self.effective_pad_len(input.len());
        let ext = extend_signal(input, edge, self.pad_type);
        let n_ext = ext.len();

        let mut forward = Vec::with_capacity(n_ext);
        self.filter.reset_to_steady_input(ext[0]);
        for x in ext.iter().copied() {
            forward.push(self.filter.process_sample(x));
        }

        self.filter.reset_to_steady_input(forward[n_ext - 1]);
        for i in 0..n_ext {
            let y = self.filter.process_sample(forward[n_ext - 1 - i]);
            let dst = n_ext - 1 - i;
            if (edge..edge + input.len()).contains(&dst) {
                output[dst - edge] = y;
            }
        }
    }
}

impl<F> ForwardBackward<F> {
    fn effective_pad_len<T>(&self, input_len: usize) -> usize
    where
        F: FiltFiltKernel<T>,
    {
        if input_len < 3 || self.pad_type == PadType::None {
            return 0;
        }

        let requested = self
            .pad_len
            .unwrap_or_else(|| self.filter.filtfilt_pad_len());
        requested.min(input_len - 2)
    }
}

fn extend_signal<T>(input: &[T], edge: usize, pad_type: PadType) -> Vec<T>
where
    T: Copy + core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
{
    if edge == 0 || pad_type == PadType::None {
        return input.to_vec();
    }

    let n = input.len();
    let first = input[0];
    let last = input[n - 1];
    let mut out = Vec::with_capacity(n + 2 * edge);

    for i in 0..edge {
        let reflected = input[edge - i];
        out.push(pad_sample(first, reflected, pad_type));
    }

    out.extend_from_slice(input);

    for i in 0..edge {
        let reflected = input[n - 2 - i];
        out.push(pad_sample(last, reflected, pad_type));
    }

    out
}

fn pad_sample<T>(endpoint: T, reflected: T, pad_type: PadType) -> T
where
    T: Copy + core::ops::Add<Output = T> + core::ops::Sub<Output = T>,
{
    match pad_type {
        PadType::Odd => endpoint + endpoint - reflected,
        PadType::Even => reflected,
        PadType::Constant => endpoint,
        PadType::None => reflected,
    }
}
