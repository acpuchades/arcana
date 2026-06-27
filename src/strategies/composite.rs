//! Composite strategies: multiple conditions combined into one entry — where the
//! signal combinators and component accessors earn their keep.

use crate::indicators::{Adx, Current, Keltner, Rsi, Sma, Value};
use crate::prelude::*;

use super::{is_flat, is_long, is_short};

/// ADX-gated moving-average crossover, long/flat.
///
/// Takes the SMA golden cross only when the trend is strong enough — ADX above
/// `adx_min` — and exits on the death cross. The strength gate uses the ADX
/// component accessor (`adx.adx()`), filtering out crossovers in chop.
pub struct AdxTrendFilter<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> AdxTrendFilter<Sym> {
    pub fn new(symbol: Sym, fast: usize, slow: usize, adx_period: usize, adx_min: Real) -> Self {
        let cross_up =
            Sma::new(Current::close(), fast).crosses_above(Sma::new(Current::close(), slow));
        Self {
            symbol,
            enter: Box::new(cross_up.and(Adx::new(adx_period).adx().above(adx_min))),
            exit: Box::new(
                Sma::new(Current::close(), fast).crosses_below(Sma::new(Current::close(), slow)),
            ),
        }
    }
}

impl<Sym: Clone> Strategy for AdxTrendFilter<Sym> {
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

/// RSI pullback within an uptrend, long/flat.
///
/// Buys an RSI dip (RSI crossing down through `oversold`) **only while** the
/// close is above its long `trend`-period SMA, so dips are bought with the trend,
/// not against it. Exits when RSI recovers up through `exit_level`.
pub struct RsiPullback<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> RsiPullback<Sym> {
    pub fn new(
        symbol: Sym,
        rsi_period: usize,
        trend: usize,
        oversold: Real,
        exit_level: Real,
    ) -> Self {
        let dip = Rsi::new(Current::close(), rsi_period).crosses_below(Value::new(oversold));
        let uptrend = Current::close().gt(Sma::new(Current::close(), trend));
        Self {
            symbol,
            enter: Box::new(dip.and(uptrend)),
            exit: Box::new(
                Rsi::new(Current::close(), rsi_period).crosses_above(Value::new(exit_level)),
            ),
        }
    }
}

impl<Sym: Clone> Strategy for RsiPullback<Sym> {
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

/// Keltner-channel breakout, always-in long/short.
///
/// An ATR-banded cousin of the Bollinger breakout: long when the close pierces
/// the upper Keltner band, short below the lower one, using the channel's
/// component accessors.
pub struct KeltnerBreakout<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> KeltnerBreakout<Sym> {
    pub fn new(symbol: Sym, ema_period: usize, atr_period: usize, multiplier: Real) -> Self {
        let channel = || Keltner::new(Current::close(), ema_period, atr_period, multiplier);
        Self {
            symbol,
            up: Box::new(Current::close().gt(channel().upper())),
            down: Box::new(Current::close().lt(channel().lower())),
        }
    }
}

impl<Sym: Clone> Strategy for KeltnerBreakout<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.up.update(candle);
        self.down.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if self.up.value() && !is_long(pos) {
            let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
        } else if self.down.value() && !is_short(pos) {
            let _ = wallet.set(self.symbol.clone(), Side::Sell, Size::value_frac(1.0));
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}
