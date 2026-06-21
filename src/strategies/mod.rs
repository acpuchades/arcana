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
//! * advances **all** of its signals/indicators on every bar before branching
//!   (never short-circuiting, or a skipped source desyncs from the price stream);
//! * sizes positions all-in via [`Size::funds_frac(1.0)`](crate::Size). Two
//!   flavours of position management appear:
//!   - **long/flat** — open on an entry edge, flatten on an exit edge;
//!   - **long/short** (always-in) — on a flip, [`close`](crate::Wallet::close)
//!     then re-open all-in on the other side (see `enter_all_in`). Flipping by
//!     flatten-then-commit keeps the all-in sizing exact, since `funds_frac`
//!     resolves against cash on hand.
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
use crate::{Side, Size, Wallet};

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

/// Flatten any existing position in `symbol`, then commit all available funds to
/// a fresh `side` position at `price`.
///
/// The flatten-then-commit shape is what makes the always-in strategies' all-in
/// sizing exact on a reversal: [`Size::funds_frac`] resolves against cash on
/// hand, so the `close` first restores full cash before the re-entry sizes.
pub(crate) fn enter_all_in<Sym: Clone>(
    wallet: &mut dyn Wallet<Sym>,
    symbol: &Sym,
    side: Side,
    price: Real,
) {
    wallet.close(symbol.clone(), price);
    wallet.open(symbol.clone(), side, Size::funds_frac(1.0), price);
}
