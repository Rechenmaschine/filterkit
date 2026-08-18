//! Combinators for composing [`SampleProcessor`]s.
//!
//! [`SampleProcessor`]: crate::traits::SampleProcessor
//! [`ProcessorExt`]: crate::traits::ProcessorExt

mod bypass;
mod chain;
mod map;
mod parallel;
mod sum;
mod tap;
mod wet_dry;

pub use bypass::Bypass;
pub use chain::Chain;
pub use map::Map;
pub use parallel::Parallel;
pub use sum::Sum;
pub use tap::Tap;
pub use wet_dry::WetDry;
