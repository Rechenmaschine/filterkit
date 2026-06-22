//! # filterkit
//!
//! Composable DSP filters and processors for embedded and desktop Rust.
//!
//! The crate is split into three layers:
//!
//! - **representation** — coefficient/parameter types describing a system
//!   ([`coeffs`]): [`FirCoeffs`], [`BiquadCoeffs`], [`SosCoeffs`],
//!   [`TransferFunction`], [`StateSpace`].
//! - **execution traits** ([`traits`]): small, shape-specific traits like
//!   [`SampleProcessor`], [`BlockProcessor`], [`StreamProcessor`],
//!   [`WholeSignalProcessor`], [`AdaptiveProcessor`].
//! - **processors** that run a representation against incoming data
//!   ([`processors`]): [`Fir`], [`Biquad`], [`SosCascade`], [`Gain`],
//!   [`Delay`], etc.
//!
//! Combinators in [`combinators`] (e.g. [`Chain`], [`Parallel`], [`WetDry`])
//! let small processors be composed into larger ones using shallow trait
//! hierarchies rather than one universal `Filter` trait.
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
//!
//! [`FirCoeffs`]: crate::coeffs::FirCoeffs
//! [`BiquadCoeffs`]: crate::coeffs::BiquadCoeffs
//! [`SosCoeffs`]: crate::coeffs::SosCoeffs
//! [`TransferFunction`]: crate::coeffs::TransferFunction
//! [`StateSpace`]: crate::coeffs::StateSpace
//! [`SampleProcessor`]: crate::traits::SampleProcessor
//! [`BlockProcessor`]: crate::traits::BlockProcessor
//! [`StreamProcessor`]: crate::traits::StreamProcessor
//! [`WholeSignalProcessor`]: crate::traits::WholeSignalProcessor
//! [`AdaptiveProcessor`]: crate::traits::AdaptiveProcessor
//! [`Fir`]: crate::processors::Fir
//! [`Biquad`]: crate::processors::Biquad
//! [`SosCascade`]: crate::processors::SosCascade
//! [`Gain`]: crate::processors::Gain
//! [`Delay`]: crate::processors::Delay
//! [`DirectFormI`]: crate::processors::DirectFormI
//! [`Chain`]: crate::combinators::Chain
//! [`Parallel`]: crate::combinators::Parallel
//! [`WetDry`]: crate::combinators::WetDry
//! [`Vec`]: alloc::vec::Vec

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_code)]
#![warn(missing_debug_implementations)]
#![allow(clippy::needless_range_loop)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod traits;
pub mod coeffs;
pub mod processors;
pub mod combinators;
pub mod stream;
pub mod whole;
pub mod adaptive;

#[cfg(feature = "design")]
pub mod design;

#[cfg(all(feature = "alloc", feature = "design"))]
pub mod response;

// Re-export the core surface for ergonomic use.
pub use traits::{
    AdaptiveProcessor, BlockProcessor, Design, Prepare, ProcessSpec, ProcessorExt, Reset, Retune,
    SampleFilter, SampleProcessor, StreamProcessor, StreamStatus, WholeSignalProcessor,
};

pub use coeffs::{BiquadCoeffs, FirCoeffs, SosCoeffs, StateSpace, TransferFunction};

pub use processors::{
    Biquad, Delay, DirectFormI, Ema, Fir, FirstOrder, FirstOrderCoeffs, Gain, SosCascade,
    StateSpaceProcessor,
};

pub use combinators::{Bypass, Chain, Map, Parallel, Sum, Tap, WetDry};
