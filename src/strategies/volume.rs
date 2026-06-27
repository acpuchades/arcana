//! Volume- and money-flow-based strategies.

use crate::indicators::{Ad, Current, Obv, Sma, Vwap};
use crate::prelude::*;

use super::is_flat;

/// On-Balance-Volume trend, long/flat.
///
/// Treats OBV crossing its own moving average as confirmation that volume is
/// backing the move: long while OBV is above its SMA, flat below it.
pub struct ObvTrend<Sym> {
    symbol: Sym,
    bullish: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> ObvTrend<Sym> {
    pub fn new(symbol: Sym, ma_period: usize) -> Self {
        Self {
            symbol,
            bullish: Box::new(Obv::new().gt(Sma::new(Obv::new(), ma_period))),
        }
    }
}

impl<Sym: Clone> Strategy for ObvTrend<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.bullish.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if self.bullish.value() && is_flat(pos) {
            let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
        } else if !self.bullish.value() && !is_flat(pos) {
            let _ = wallet.close(self.symbol.clone());
        }
    }

    fn reset(&mut self) {
        self.bullish.reset();
    }
}

/// VWAP reversion, long/flat.
///
/// Buys when price dips below the (session-anchored) VWAP and exits when it
/// recovers above — a classic intraday "fair value" fade. Call
/// [`reset`](Strategy::reset) at each session boundary to re-anchor the VWAP.
pub struct VwapReversion<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> VwapReversion<Sym> {
    pub fn new(symbol: Sym) -> Self {
        Self {
            symbol,
            enter: Box::new(Current::close().crosses_below(Vwap::new())),
            exit: Box::new(Current::close().crosses_above(Vwap::new())),
        }
    }
}

impl<Sym: Clone> Strategy for VwapReversion<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.enter.update(candle);
        self.exit.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if self.enter.value() && is_flat(pos) {
            let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
        } else if self.exit.value() && !is_flat(pos) {
            let _ = wallet.close(self.symbol.clone());
        }
    }

    fn reset(&mut self) {
        self.enter.reset();
        self.exit.reset();
    }
}

/// Chaikin Accumulation/Distribution trend, long/flat.
///
/// Like [`ObvTrend`] but on the Chaikin A/D line, which weights each bar's
/// volume by where the close fell within its range: long while the A/D line is
/// above its moving average, flat below.
pub struct ChaikinAdTrend<Sym> {
    symbol: Sym,
    bullish: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> ChaikinAdTrend<Sym> {
    pub fn new(symbol: Sym, ma_period: usize) -> Self {
        Self {
            symbol,
            bullish: Box::new(Ad::new().gt(Sma::new(Ad::new(), ma_period))),
        }
    }
}

impl<Sym: Clone> Strategy for ChaikinAdTrend<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.bullish.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if self.bullish.value() && is_flat(pos) {
            let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
        } else if !self.bullish.value() && !is_flat(pos) {
            let _ = wallet.close(self.symbol.clone());
        }
    }

    fn reset(&mut self) {
        self.bullish.reset();
    }
}
