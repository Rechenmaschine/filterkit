//! # filterkit
//!
//! Composable DSP filters and processors for Rust.
//!
//! Filter representations and runtime state are separate. Processing is
//! grouped into sample, block, stream, whole-signal, and adaptive traits.
//!
//! ## API index
//!
//! | Module | Main items |
//! | --- | --- |
//! | [`traits`] | Processing and lifecycle traits |
//! | [`coeffs`] | `BiquadCoeffs`, `FirCoeffs`, `SosCoeffs`, `StateSpace`, `TransferFunction` |
//! | [`processors`] | `Biquad`, `Fir`, `Ema`, `FirstOrder`, `DirectFormI`, `Gain`, `Delay`, `SosCascade` |
//! | [`combinators`] | `Chain`, `Parallel`, `Sum`, `Map`, `Tap`, `Bypass`, `WetDry` |
//! | [`stream`] | `Decimator`, `Interpolator`, `PolyphaseResampler` |
//! | [`whole`] | `ForwardBackward`, `PadType` |
//! | [`adaptive`] | `Lms`, `Nlms` |
//! | `design` | Biquad, EMA, moving-average, and windowed-sinc specifications |
//! | `response` | Frequency-response, sweep, impulse-response, and step-response helpers |
//! | `estimators` | `GaussianEstimate`, `KalmanModel`, `KalmanFilter` |
//!
//! `design`, `response`, and `estimators` are feature-gated. With
//! `default-features = false`, the crate uses no heap allocation.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]
#![allow(clippy::needless_range_loop)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod adaptive;
pub mod coeffs;
pub mod combinators;
pub mod processors;
pub mod stream;
pub mod traits;
pub mod whole;

#[cfg(feature = "kalman")]
pub mod estimators;

#[cfg(feature = "design")]
pub mod design;

#[cfg(all(feature = "alloc", feature = "design"))]
pub mod response;

pub use traits::{
    AdaptiveProcessor, BlockProcessor, Design, Prepare, ProcessSpec, ProcessorExt, Reset, Retune,
    SampleProcessor, StreamProcessor, StreamStatus, WholeSignalProcessor,
};

pub use coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs, StateSpace, TransferFunction};

pub use processors::{
    Biquad, Delay, DirectFormI, Ema, Fir, FirstOrder, FirstOrderCoeffs, Gain, SosCascade,
    StateSpaceProcessor,
};

pub use combinators::{Bypass, Chain, Map, Parallel, Sum, Tap, WetDry};

#[cfg(feature = "kalman")]
pub use estimators::{GaussianEstimate, KalmanFilter, KalmanModel};
