//! Behavioural tests for the built-in strategy catalogue: every strategy is run,
//! one bar at a time, into a `PaperWallet` over a synthetic price path that both
//! trends (up then down) and oscillates — so trend, breakout, mean-reversion,
//! momentum, volume and composite strategies all find something to trade.

use arcana::prelude::*;
use arcana::strategies::composite::{AdxTrendFilter, KeltnerBreakout, RsiPullback};
use arcana::strategies::mean_reversion::{
    BollingerReversion, MfiReversal, RsiReversal, StochRsiReversal, StochasticReversal,
    ZScoreReversion,
};
use arcana::strategies::momentum::{MomentumRoc, RsiMidline};
use arcana::strategies::trend::{
    BollingerBreakout, DonchianBreakout, MacdCrossover, MacdZeroCross, MaCrossover, TripleMa,
};
use arcana::strategies::volume::{ChaikinAdTrend, ObvTrend, VwapReversion};

const SYMBOL: &str = "X";
const FUNDS: Real = 10_000.0;

/// A price path that rises for the first half and falls for the second, with a
/// steady oscillation on top — rich enough to exercise every strategy family.
fn series() -> Vec<Candle> {
    let mut candles = Vec::new();
    let mut prev_close: Real = 100.0;
    for i in 0..200i32 {
        let trend = if i < 100 {
            100.0 + f64::from(i) * 0.8
        } else {
            180.0 - f64::from(i - 100) * 0.8
        };
        let close = trend + 10.0 * (f64::from(i) * 0.25).sin();
        let open = prev_close;
        let high = open.max(close) + 0.75;
        let low = open.min(close) - 0.75;
        // Volume scales with the size of the move (regardless of direction), so
        // money-flow indicators reach their extremes on the steep swings while
        // OBV/AD still read trend from the sign of each bar.
        let volume = 1_000.0 + 200.0 * (close - open).abs();
        candles.push(Candle::new(open, high, low, close, volume));
        prev_close = close;
    }
    candles
}

/// Drive `strat` over `candles` into a fresh wallet and hand it back.
fn run<S>(mut strat: S, candles: &[Candle]) -> PaperWallet<&'static str>
where
    S: Strategy<Input = Candle, Symbol = &'static str>,
{
    let mut wallet = PaperWallet::new(FUNDS);
    for &candle in candles {
        wallet.update(SYMBOL, Reference(candle.close));
        strat.update(candle);
        strat.trade(&mut wallet);
    }
    wallet
}

/// Assert a strategy actually traded, and left the wallet in a finite state.
fn assert_trades<S>(name: &str, strat: S, candles: &[Candle])
where
    S: Strategy<Input = Candle, Symbol = &'static str>,
{
    let wallet = run(strat, candles);
    assert!(!wallet.orders().is_empty(), "{name} never traded");
    assert!(wallet.funds().0.is_finite(), "{name} produced non-finite funds");
}

#[test]
fn every_strategy_trades_over_the_path() {
    let c = series();

    // Trend-following.
    assert_trades("MaCrossover", MaCrossover::new(SYMBOL, 5, 20), &c);
    assert_trades("MacdCrossover", MacdCrossover::new(SYMBOL, 12, 26, 9), &c);
    assert_trades("MacdZeroCross", MacdZeroCross::new(SYMBOL, 12, 26, 9), &c);
    assert_trades("DonchianBreakout", DonchianBreakout::new(SYMBOL, 20), &c);
    assert_trades("TripleMa", TripleMa::new(SYMBOL, 5, 10, 20), &c);
    assert_trades("BollingerBreakout", BollingerBreakout::new(SYMBOL, 20, 2.0), &c);

    // Mean-reversion.
    assert_trades("RsiReversal", RsiReversal::new(SYMBOL, 14, 30.0, 50.0), &c);
    assert_trades("BollingerReversion", BollingerReversion::new(SYMBOL, 20, 2.0), &c);
    assert_trades("StochasticReversal", StochasticReversal::new(SYMBOL, 14, 0.2, 0.8), &c);
    assert_trades("StochRsiReversal", StochRsiReversal::new(SYMBOL, 14, 14, 0.2, 0.8), &c);
    assert_trades("MfiReversal", MfiReversal::new(SYMBOL, 14, 20.0, 80.0), &c);
    assert_trades("ZScoreReversion", ZScoreReversion::new(SYMBOL, 20, 1.0), &c);

    // Momentum.
    assert_trades("MomentumRoc", MomentumRoc::new(SYMBOL, 10), &c);
    assert_trades("RsiMidline", RsiMidline::new(SYMBOL, 14), &c);

    // Volume / flow.
    assert_trades("ObvTrend", ObvTrend::new(SYMBOL, 20), &c);
    assert_trades("VwapReversion", VwapReversion::new(SYMBOL), &c);
    assert_trades("ChaikinAdTrend", ChaikinAdTrend::new(SYMBOL, 20), &c);

    // Composite.
    assert_trades("AdxTrendFilter", AdxTrendFilter::new(SYMBOL, 5, 20, 14, 10.0), &c);
    // A Connors-style short-period RSI: a 14-period RSI rarely pulls back to
    // oversold mid-uptrend, but RSI(2) dips hard on any down-bar.
    assert_trades("RsiPullback", RsiPullback::new(SYMBOL, 2, 20, 15.0, 60.0), &c);
    assert_trades("KeltnerBreakout", KeltnerBreakout::new(SYMBOL, 20, 10, 2.0), &c);
}

#[test]
fn ma_crossover_goes_long_then_short() {
    // A clean rise then fall gives one golden cross (Buy) followed by a death
    // cross that reverses to short (Sell).
    let mut prices: Vec<Real> = (0..15).map(|i| 100.0 + f64::from(i)).collect();
    prices.extend((0..15).map(|i| 115.0 - f64::from(i) * 2.0));
    let candles: Vec<Candle> = prices.iter().map(|&p| Candle::new(p, p, p, p, 1.0)).collect();

    let wallet = run(MaCrossover::new(SYMBOL, 3, 8), &candles);
    let sides: Vec<Side> = wallet.orders().iter().map(|o| o.side).collect();
    assert_eq!(sides.first(), Some(&Side::Buy), "first action is the golden cross");
    assert!(sides.contains(&Side::Sell), "the death cross reverses to short");
}

#[test]
fn rsi_reversal_buys_the_dip_and_exits_flat() {
    // Sell off into oversold, then recover through the exit level.
    let mut prices: Vec<Real> = (0..14).map(|i| 100.0 - f64::from(i) * 3.0).collect();
    prices.extend((0..14).map(|i| 60.0 + f64::from(i) * 3.0));
    let candles: Vec<Candle> = prices.iter().map(|&p| Candle::new(p, p, p, p, 1.0)).collect();

    let wallet = run(RsiReversal::new(SYMBOL, 5, 30.0, 50.0), &candles);
    assert!(!wallet.orders().is_empty(), "should have bought the dip");
    assert!(wallet.is_flat(), "should have exited on the recovery");
    let sides: Vec<Side> = wallet.orders().iter().map(|o| o.side).collect();
    assert_eq!(sides.first(), Some(&Side::Buy));
    assert_eq!(sides.last(), Some(&Side::Sell));
}

#[test]
fn reset_returns_a_strategy_to_its_initial_state() {
    let c = series();
    let mut strat = MaCrossover::new(SYMBOL, 5, 20);

    let mut first = PaperWallet::new(FUNDS);
    for &candle in &c {
        first.update(SYMBOL, Reference(candle.close));
        strat.update(candle);
        strat.trade(&mut first);
    }

    strat.reset();
    let mut second = PaperWallet::new(FUNDS);
    for &candle in &c {
        second.update(SYMBOL, Reference(candle.close));
        strat.update(candle);
        strat.trade(&mut second);
    }

    // After reset the strategy replays identically.
    assert_eq!(first.orders(), second.orders());
}
