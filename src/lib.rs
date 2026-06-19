//! # Arcana
//!
//! A library of technical-analysis (TA) building blocks designed around
//! *incremental* computation. Every primitive owns its internal state and is
//! advanced one sample at a time through `update()`, carrying just enough
//! intermediate state to produce the next output in O(1) (or close to it).
//! This makes the same code usable for live streaming and batch backtesting.
//!
//! The crate has two composable layers:
//!
//! * [`Indicator`] — the numeric *sources*. Each incrementally produces a
//!   [`Real`] and **owns its own input source**, so composition is just nesting
//!   constructors: `Ema::new(Current::close(), 20)` is the EMA-20 of the close,
//!   `Ema::new(Sma::new(src, 10), 20)` an EMA of an SMA. Outputs are exposed as
//!   public fields refreshed every [`Indicator::update`]; a single-output
//!   indicator exposes a field named `value`. Leaf sources ([`Value`] for a
//!   constant, [`Identity`] for the raw input, `Current::*` for candle fields)
//!   terminate the chain. Bar indicators ([`Atr`](crate::indicators::Atr),
//!   [`Adx`](crate::indicators::Adx)) consume a [`Candle`] directly.
//! * [`Signal`] — incremental, *composable* booleans. Comparison signals are
//!   built from two sources, so a condition like "RSI over 70" is a single
//!   object; combine them further with the [`SignalExt`] combinators
//!   (`and`/`or`/`xor`/`not`/`changed`).
//!
//! ```
//! use arcana::prelude::*;
//! use arcana::indicators::{Identity, Rsi};
//!
//! // "RSI(14) over 70" as a single Signal<Input = Real>. Indicators own their
//! // source, so `Identity` feeds the raw price stream into the RSI.
//! let mut overbought = Rsi::new(Identity::new(), 14).above(70.0);
//! for price in [44.0, 44.3, 44.1, 43.6, 44.3, 44.8, 45.1, 45.6] {
//!     overbought.update(price);
//! }
//! let _ = overbought.value();
//! ```
//!
//! [`Value`]: crate::indicators::Value
//! [`Identity`]: crate::indicators::Identity

pub mod indicator;
pub mod indicators;
pub mod signal;
pub mod signals;
pub mod types;

pub use indicator::Indicator;
pub use signal::{Signal, SignalExt};
pub use types::{Candle, Real};

/// Convenient glob-import of the core traits and types.
pub mod prelude {
    pub use crate::indicator::Indicator;
    pub use crate::signal::{Signal, SignalExt};
    pub use crate::signals::IndicatorExt;
    pub use crate::types::{Candle, Real};
}
