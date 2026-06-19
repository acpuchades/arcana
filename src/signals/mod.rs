//! Concrete signal implementations.
//!
//! Comparison signals ([`Gt`], [`Lt`], [`Ge`], [`Le`], [`Eq`], [`Ne`]) are
//! built from two composable indicator *sources*, so a condition like "RSI over
//! 70" is a single object. Build them fluently with [`IndicatorExt`]
//! (`a.lt(b)`, `rsi.above(70.0)`, `fast.crosses_above(slow)`) and combine the
//! results with the [`SignalExt`](crate::SignalExt) combinators
//! (`and`/`or`/`xor`/`not`/`changed`).
//!
//! A *crossover* is not a primitive: it composes as "the comparison is true and
//! it just changed", which [`crosses_above`](IndicatorExt::crosses_above)
//! builds for you. All comparisons are tolerance-aware (see [`DEFAULT_EPSILON`]).

pub mod compare;

pub use compare::{Compare, CompareOp, DEFAULT_EPSILON, Eq, Ge, Gt, IndicatorExt, Le, Lt, Ne};
