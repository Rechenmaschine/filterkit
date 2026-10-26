//! # filterkit
//!
//! Composable DSP filters and processors for Rust.
//!
//! Filter representations and runtime state are separate. Processing is
//! grouped into sample and whole-signal traits.
//!
//! ## API index
//!
//! | Module | Main items |
//! | --- | --- |
//! | [`traits`] | Processing and lifecycle traits |
//! | [`coeffs`] | `BiquadCoeffs`, `FirCoeffs`, `FirstOrderCoeffs`, `SosCoeffs`, `StateSpace`, `TransferFunction` |
//! | [`processors`] | `Biquad`, `Fir`, `Ema`, `FirstOrder`, `DirectFormI`, `Gain`, `Delay`, `SosCascade` |
//! | [`combinators`] | `Chain`, `Parallel`, `Sum`, `Map`, `Tap`, `Bypass`, `WetDry` |
//! | [`whole`] | `ForwardBackward`, `PadType` |
//! | `design` | Biquad, EMA, moving-average, and windowed-sinc specifications |
//! | `estimators` | `GaussianEstimate`, `KalmanModel`, `KalmanFilter` |
//!
//! `design` and `estimators` are feature-gated. With
//! `default-features = false`, the crate uses no heap allocation.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]
#![allow(clippy::needless_range_loop)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod coeffs;
pub mod combinators;
pub mod processors;
pub mod traits;
pub mod whole;

#[cfg(feature = "kalman")]
pub mod estimators;

#[cfg(feature = "design")]
pub mod design;

pub use traits::{ProcessorExt, Reset, Retune, SampleProcessor, WholeSignalProcessor};

pub use coeffs::{
    BiquadCoeffs, FirCoeffs, FirstOrderCoeffs, SosCoeffs, StateSpace, TransferFunction,
};

pub use processors::{
    Biquad, Delay, DirectFormI, Ema, Fir, FirstOrder, Gain, SosCascade, StateSpaceProcessor,
};

pub use combinators::{Bypass, Chain, Map, Parallel, Sum, Tap, WetDry};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(feature = "kalman")]
pub use estimators::{GaussianEstimate, KalmanFilter, KalmanModel};

/// The `nalgebra` version used by [`KalmanModel`].
#[cfg(feature = "kalman")]
pub use nalgebra;
