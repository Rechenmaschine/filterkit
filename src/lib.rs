//! # filterkit
//!
//! Composable DSP filters and processors for embedded and desktop Rust.
//!
//! It provides filter representations, processing traits, stateful
//! processors, combinators, and optional design and analysis helpers.
//!
//! ## Features
//!
//! | Feature   | Effect                                                          |
//! |-----------|-----------------------------------------------------------------|
//! | `std`     | Pulls in `std` (default). Implies `alloc`.                      |
//! | `alloc`   | Enables [`Vec`]-backed dynamic processors like `FirDyn`.        |
//! | `design`  | Enables high-level design helpers (windowed sinc, biquad, …).   |
//!
//! With `default-features = false`, the crate is `no_std`-compatible and
//! allocates nothing on the heap.

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
