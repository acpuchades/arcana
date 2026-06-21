//! Trend-following strategies: crossover and breakout entries that ride a move.

use crate::indicators::{Bollinger, Current, Donchian, Macd, Sma, Value};
use crate::prelude::*;

use super::{enter_all_in, is_flat, is_long, is_short};

/// Moving-average crossover (the "golden / death cross"), always-in long/short.
///
/// Goes long when the fast SMA crosses above the slow SMA and reverses to short
/// on the opposite cross, always committing all funds to the prevailing side.
pub struct MaCrossover<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> MaCrossover<Sym> {
    pub fn new(symbol: Sym, fast: usize, slow: usize) -> Self {
        Self {
            symbol,
            up: Box::new(
                Sma::new(Current::close(), fast).crosses_above(Sma::new(Current::close(), slow)),
            ),
            down: Box::new(
                Sma::new(Current::close(), fast).crosses_below(Sma::new(Current::close(), slow)),
            ),
        }
    }
}

impl<Sym: Clone> Strategy for MaCrossover<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let up = self.up.update(candle);
        let down = self.down.update(candle);
        let pos = wallet.position(&self.symbol);
        if up && !is_long(pos) {
            enter_all_in(wallet, &self.symbol, Side::Buy, candle.close);
        } else if down && !is_short(pos) {
            enter_all_in(wallet, &self.symbol, Side::Sell, candle.close);
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}

/// MACD line / signal-line crossover, always-in long/short.
///
/// Long when the MACD line crosses above its signal line, short on the opposite
/// cross. Built straight from the MACD component accessors.
pub struct MacdCrossover<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> MacdCrossover<Sym> {
    pub fn new(symbol: Sym, fast: usize, slow: usize, signal: usize) -> Self {
        let macd = Macd::new(Current::close(), fast, slow, signal);
        Self {
            symbol,
            up: Box::new(macd.line().crosses_above(macd.signal())),
            down: Box::new(macd.line().crosses_below(macd.signal())),
        }
    }
}

impl<Sym: Clone> Strategy for MacdCrossover<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let up = self.up.update(candle);
        let down = self.down.update(candle);
        let pos = wallet.position(&self.symbol);
        if up && !is_long(pos) {
            enter_all_in(wallet, &self.symbol, Side::Buy, candle.close);
        } else if down && !is_short(pos) {
            enter_all_in(wallet, &self.symbol, Side::Sell, candle.close);
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}

/// MACD zero-line crossover, always-in long/short.
///
/// A pure momentum-of-momentum read: long while the MACD line is above zero
/// (fast EMA over slow), short below it, flipping on the zero crossing.
pub struct MacdZeroCross<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> MacdZeroCross<Sym> {
    pub fn new(symbol: Sym, fast: usize, slow: usize, signal: usize) -> Self {
        let macd = Macd::new(Current::close(), fast, slow, signal);
        Self {
            symbol,
            up: Box::new(macd.line().crosses_above(Value::new(0.0))),
            down: Box::new(macd.line().crosses_below(Value::new(0.0))),
        }
    }
}

impl<Sym: Clone> Strategy for MacdZeroCross<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let up = self.up.update(candle);
        let down = self.down.update(candle);
        let pos = wallet.position(&self.symbol);
        if up && !is_long(pos) {
            enter_all_in(wallet, &self.symbol, Side::Buy, candle.close);
        } else if down && !is_short(pos) {
            enter_all_in(wallet, &self.symbol, Side::Sell, candle.close);
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}

/// Donchian-channel breakout (the classic Turtle entry), always-in long/short.
///
/// Long when the close breaks above the highest high of the prior `period` bars,
/// short when it breaks below the prior `period`-bar low. The channel is lagged
/// one bar so the breakout is measured against the *prior* channel, not one that
/// already contains the breakout bar.
pub struct DonchianBreakout<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> DonchianBreakout<Sym> {
    pub fn new(symbol: Sym, period: usize) -> Self {
        let channel = || Donchian::new(Current::high(), Current::low(), period);
        Self {
            symbol,
            up: Box::new(Current::close().gt(channel().upper().lag(1))),
            down: Box::new(Current::close().lt(channel().lower().lag(1))),
        }
    }
}

impl<Sym: Clone> Strategy for DonchianBreakout<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let up = self.up.update(candle);
        let down = self.down.update(candle);
        let pos = wallet.position(&self.symbol);
        if up && !is_long(pos) {
            enter_all_in(wallet, &self.symbol, Side::Buy, candle.close);
        } else if down && !is_short(pos) {
            enter_all_in(wallet, &self.symbol, Side::Sell, candle.close);
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}

/// Triple moving-average alignment, long/flat.
///
/// Holds a long position only while the three SMAs are stacked bullishly
/// (`fast > mid > slow`), flattening as soon as that alignment breaks.
pub struct TripleMa<Sym> {
    symbol: Sym,
    aligned: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> TripleMa<Sym> {
    pub fn new(symbol: Sym, fast: usize, mid: usize, slow: usize) -> Self {
        Self {
            symbol,
            aligned: Box::new(
                Sma::new(Current::close(), fast)
                    .gt(Sma::new(Current::close(), mid))
                    .and(Sma::new(Current::close(), mid).gt(Sma::new(Current::close(), slow))),
            ),
        }
    }
}

impl<Sym: Clone> Strategy for TripleMa<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let aligned = self.aligned.update(candle);
        let pos = wallet.position(&self.symbol);
        if aligned && is_flat(pos) {
            wallet.open(self.symbol.clone(), Side::Buy, Size::funds_frac(1.0), candle.close);
        } else if !aligned && !is_flat(pos) {
            wallet.close(self.symbol.clone(), candle.close);
        }
    }

    fn reset(&mut self) {
        self.aligned.reset();
    }
}

/// Bollinger-band breakout, always-in long/short.
///
/// Treats a close beyond a band as momentum: long above the upper band, short
/// below the lower one. (Contrast [`BollingerReversion`](super::mean_reversion::BollingerReversion),
/// which fades the same bands.)
pub struct BollingerBreakout<Sym> {
    symbol: Sym,
    up: Box<dyn Signal<Input = Candle>>,
    down: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> BollingerBreakout<Sym> {
    pub fn new(symbol: Sym, period: usize, k: Real) -> Self {
        let bands = Bollinger::new(Current::close(), period, k);
        Self {
            symbol,
            up: Box::new(Current::close().gt(bands.upper())),
            down: Box::new(Current::close().lt(bands.lower())),
        }
    }
}

impl<Sym: Clone> Strategy for BollingerBreakout<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn evaluate(&mut self, candle: Candle, wallet: &mut dyn Wallet<Sym>) {
        let up = self.up.update(candle);
        let down = self.down.update(candle);
        let pos = wallet.position(&self.symbol);
        if up && !is_long(pos) {
            enter_all_in(wallet, &self.symbol, Side::Buy, candle.close);
        } else if down && !is_short(pos) {
            enter_all_in(wallet, &self.symbol, Side::Sell, candle.close);
        }
    }

    fn reset(&mut self) {
        self.up.reset();
        self.down.reset();
    }
}
