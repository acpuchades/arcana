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

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let bullish = self.bullish.update(candle);
        let pos = wallet.position(&self.symbol);
        if bullish && is_flat(pos) {
            wallet.open(self.symbol.clone(), Side::Buy, Size::funds_frac(1.0), candle.close);
        } else if !bullish && !is_flat(pos) {
            wallet.close(self.symbol.clone(), candle.close);
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

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let enter = self.enter.update(candle);
        let exit = self.exit.update(candle);
        let pos = wallet.position(&self.symbol);
        if enter && is_flat(pos) {
            wallet.open(self.symbol.clone(), Side::Buy, Size::funds_frac(1.0), candle.close);
        } else if exit && !is_flat(pos) {
            wallet.close(self.symbol.clone(), candle.close);
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

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let bullish = self.bullish.update(candle);
        let pos = wallet.position(&self.symbol);
        if bullish && is_flat(pos) {
            wallet.open(self.symbol.clone(), Side::Buy, Size::funds_frac(1.0), candle.close);
        } else if !bullish && !is_flat(pos) {
            wallet.close(self.symbol.clone(), candle.close);
        }
    }

    fn reset(&mut self) {
        self.bullish.reset();
    }
}
