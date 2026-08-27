//! Combinators for composing [`crate::traits::SampleProcessor`]s.

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
