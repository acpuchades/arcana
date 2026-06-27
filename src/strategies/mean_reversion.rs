//! Mean-reversion strategies: fade an extreme, exit as price returns to normal.

use crate::indicators::{Bollinger, Current, Mfi, Rsi, Sma, StdDev, Stochastic, Value};
use crate::prelude::*;

use super::{is_flat, is_long, is_short};

/// RSI oversold-bounce, long/flat.
///
/// Buys the dip when RSI crosses *down* through `oversold`, and exits when RSI
/// recovers up through `exit_level` (e.g. 30 → 50).
pub struct RsiReversal<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> RsiReversal<Sym> {
    pub fn new(symbol: Sym, period: usize, oversold: Real, exit_level: Real) -> Self {
        Self {
            symbol,
            enter: Box::new(Rsi::new(Current::close(), period).crosses_below(Value::new(oversold))),
            exit: Box::new(Rsi::new(Current::close(), period).crosses_above(Value::new(exit_level))),
        }
    }
}

impl<Sym: Clone> Strategy for RsiReversal<Sym> {
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

/// Bollinger-band reversion, long/flat.
///
/// Buys when the close crosses below the lower band and exits when it crosses
/// back above the middle band (the moving average). Fades the bands rather than
/// chasing the breakout.
pub struct BollingerReversion<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> BollingerReversion<Sym> {
    pub fn new(symbol: Sym, period: usize, k: Real) -> Self {
        let bands = Bollinger::new(Current::close(), period, k);
        Self {
            symbol,
            enter: Box::new(Current::close().crosses_below(bands.lower())),
            exit: Box::new(Current::close().crosses_above(bands.middle())),
        }
    }
}

impl<Sym: Clone> Strategy for BollingerReversion<Sym> {
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

/// Stochastic oscillator oversold-bounce, long/flat.
///
/// The stochastic ranges `0..1` here, so `oversold`/`overbought` are fractions
/// (e.g. 0.2 / 0.8). Buys when %K crosses down through `oversold`, exits when it
/// crosses up through `overbought`.
pub struct StochasticReversal<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> StochasticReversal<Sym> {
    pub fn new(symbol: Sym, period: usize, oversold: Real, overbought: Real) -> Self {
        Self {
            symbol,
            enter: Box::new(
                Stochastic::new(Current::close(), period).crosses_below(Value::new(oversold)),
            ),
            exit: Box::new(
                Stochastic::new(Current::close(), period).crosses_above(Value::new(overbought)),
            ),
        }
    }
}

impl<Sym: Clone> Strategy for StochasticReversal<Sym> {
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

/// StochRSI oversold-bounce, long/flat.
///
/// The stochastic transform over an RSI source (also `0..1`): a more responsive
/// oscillator than either alone. Same dip-buy / recovery-exit edges as
/// [`StochasticReversal`].
pub struct StochRsiReversal<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> StochRsiReversal<Sym> {
    pub fn new(
        symbol: Sym,
        rsi_period: usize,
        stoch_period: usize,
        oversold: Real,
        overbought: Real,
    ) -> Self {
        let stoch_rsi =
            || Stochastic::new(Rsi::new(Current::close(), rsi_period), stoch_period);
        Self {
            symbol,
            enter: Box::new(stoch_rsi().crosses_below(Value::new(oversold))),
            exit: Box::new(stoch_rsi().crosses_above(Value::new(overbought))),
        }
    }
}

impl<Sym: Clone> Strategy for StochRsiReversal<Sym> {
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

/// Money-Flow-Index oversold-bounce, long/flat.
///
/// Volume-weighted RSI cousin (`0..100`): buys when MFI crosses down through
/// `oversold`, exits when it crosses up through `overbought` (e.g. 20 / 80).
pub struct MfiReversal<Sym> {
    symbol: Sym,
    enter: Box<dyn Signal<Input = Candle>>,
    exit: Box<dyn Signal<Input = Candle>>,
}

impl<Sym> MfiReversal<Sym> {
    pub fn new(symbol: Sym, period: usize, oversold: Real, overbought: Real) -> Self {
        Self {
            symbol,
            enter: Box::new(Mfi::new(period).crosses_below(Value::new(oversold))),
            exit: Box::new(Mfi::new(period).crosses_above(Value::new(overbought))),
        }
    }
}

impl<Sym: Clone> Strategy for MfiReversal<Sym> {
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

/// Z-score reversion, always-in long/short.
///
/// Trades the standardised deviation of price from its mean,
/// `z = (close − SMA) / StdDev`: long when `z ≤ −entry` (cheap), short when
/// `z ≥ entry` (rich), and flattening once `z` reverts back through zero. Built
/// by composing the arithmetic operators over the close, its SMA and its StdDev.
pub struct ZScoreReversion<Sym> {
    symbol: Sym,
    z: Box<dyn Indicator<Input = Candle, Output = Real>>,
    entry: Real,
}

impl<Sym> ZScoreReversion<Sym> {
    pub fn new(symbol: Sym, period: usize, entry: Real) -> Self {
        Self {
            symbol,
            z: Box::new(
                Current::close()
                    .sub(Sma::new(Current::close(), period))
                    .div(StdDev::new(Current::close(), period)),
            ),
            entry,
        }
    }
}

impl<Sym: Clone> Strategy for ZScoreReversion<Sym> {
    type Input = Candle;
    type Symbol = Sym;

    fn update(&mut self, candle: Candle) {
        self.z.update(candle);
    }

    fn trade(&self, wallet: &mut dyn Wallet<Sym>) {
        let pos = wallet.position(&self.symbol).amount;
        if let Some(z) = self.z.current() {
            if z <= -self.entry && !is_long(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Buy, Size::value_frac(1.0));
            } else if z >= self.entry && !is_short(pos) {
                let _ = wallet.set(self.symbol.clone(), Side::Sell, Size::value_frac(1.0));
            } else if (is_long(pos) && z >= 0.0) || (is_short(pos) && z <= 0.0) {
                let _ = wallet.close(self.symbol.clone());
            }
        }
    }

    fn reset(&mut self) {
        self.z.reset();
    }
}
