//! A catalogue of **classical single-asset strategies**, each a concrete
//! [`Strategy`](crate::Strategy) type ready to trade into a [`Wallet`].
//!
//! These exist as a convenience library of worked examples — every one is just
//! "the user's own type implementing the trait" (a struct holding its signals or
//! indicators, whose [`evaluate`](crate::Strategy::evaluate) reads the bar and
//! calls wallet methods). There is **no rule engine, policy object, or
//! `(signal, action)` table** here; each strategy spells out its own decision
//! logic, exactly as a downstream user would write it.
//!
//! Every strategy:
//!
//! * is generic over the symbol type `Sym: Clone` and takes `Input = Candle`;
//! * in [`update`](crate::Strategy::update) advances **all** of its
//!   signals/indicators every bar (never short-circuiting, or a skipped source
//!   desyncs from the price stream), then decides in [`trade`](crate::Strategy::trade);
//! * sizes positions all-in via [`Size::value_frac(1.0)`](crate::Size). Two
//!   flavours of position management appear:
//!   - **long/flat** — [`set`](crate::Wallet::set) all-in on an entry edge,
//!     [`close`](crate::Wallet::close) on an exit edge;
//!   - **long/short** (always-in) — flip with a single
//!     [`set`](crate::Wallet::set) to the other side. Because `value_frac`
//!     resolves against equity (which survives a reversal, unlike cash), one
//!     `set` reverses and re-sizes all-in exactly — no flatten-then-reopen.
//!
//! The families:
//!
//! * [`trend`] — crossover / breakout trend-following.
//! * [`mean_reversion`] — oscillator and band reversion.
//! * [`momentum`] — rate-of-change / oscillator-vs-midline.
//! * [`volume`] — volume- and flow-based.
//! * [`composite`] — multi-condition (trend gated by strength, dip-in-uptrend).

pub mod composite;
pub mod mean_reversion;
pub mod momentum;
pub mod trend;
pub mod volume;

use crate::signals::DEFAULT_EPSILON;
use crate::types::Real;

/// Whether `position` is effectively flat (within [`DEFAULT_EPSILON`]).
pub(crate) fn is_flat(position: Real) -> bool {
    position.abs() <= DEFAULT_EPSILON
}

/// Whether `position` is meaningfully long.
pub(crate) fn is_long(position: Real) -> bool {
    position > DEFAULT_EPSILON
}

/// Whether `position` is meaningfully short.
pub(crate) fn is_short(position: Real) -> bool {
    position < -DEFAULT_EPSILON
}
