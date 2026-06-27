//! Momentum strategies: trade the sign of a rate-of-change or oscillator.

use crate::indicators::{Close, Current, Roc, Rsi};
use crate::prelude::*;

use super::{is_long, is_short};

/// Rate-of-change momentum, always-in long/short.
///
/// Long while the `period`-bar percentage change of the close is positive, short
/// while it is negative — the simplest time-series momentum rule.
pub struct MomentumRoc<Sym> {
    symbol: Sym,
    roc: Roc<Close>,
}

impl<Sym> MomentumRoc<Sym> {
    pub fn new(symbol: Sym, period: usize) -> Self {
        Self {
            symbol,
            roc: Current::close().roc(period),
        }
    }
}

impl<Sym: Clone> Strategy for MomentumRoc<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.roc.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if let Some(roc) = self.roc.current() {
            if roc > 0.0 && !is_long(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
            } else if roc < 0.0 && !is_short(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Sell, Size::value_frac(1.0));
            }
        }
    }

    fn reset(&mut self) {
        self.roc.reset();
    }
}

/// RSI midline momentum, always-in long/short.
///
/// Reads RSI as a trend gauge rather than a reversion one: long while RSI is
/// above 50, short while below — flipping as it crosses the midline.
pub struct RsiMidline<Sym> {
    symbol: Sym,
    rsi: Rsi<Close>,
}

impl<Sym> RsiMidline<Sym> {
    pub fn new(symbol: Sym, period: usize) -> Self {
        Self {
            symbol,
            rsi: Rsi::new(Current::close(), period),
        }
    }
}

impl<Sym: Clone> Strategy for RsiMidline<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.rsi.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if let Some(rsi) = self.rsi.current() {
            if rsi > 50.0 && !is_long(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
            } else if rsi < 50.0 && !is_short(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Sell, Size::value_frac(1.0));
            }
        }
    }

    fn reset(&mut self) {
        self.rsi.reset();
    }
}
