use crate::indicator::Indicator;
use crate::indicators::smoothing::WilderState;
use crate::types::{Candle, Real};

/// The directional outputs of [`Adx`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdxValue {
    /// Positive directional indicator, `+DI`.
    pub plus_di: Real,
    /// Negative directional indicator, `-DI`.
    pub minus_di: Real,
    /// The average directional index, `ADX`.
    pub adx: Real,
}

/// Average Directional Index (Wilder).
///
/// A bar indicator (consumes the full [`Candle`]). Tracks directional movement:
/// `+DM`, `-DM` and true range are each Wilder-smoothed to form `+DI` / `-DI`;
/// their normalised spread `DX` is smoothed again to produce `ADX`.
///
/// `+DI` and `-DI` become available after `period` directional bars; `adx`
/// follows after a further `period` bars. The directional fields are exposed
/// individually; [`current`](Indicator::current) / [`update`](Indicator::update)
/// only yield a value once `adx` itself is ready.
#[derive(Debug, Clone)]
pub struct Adx {
    // Previous bar's high, low and close.
    prev: Option<(Real, Real, Real)>,
    plus_dm: WilderState,
    minus_dm: WilderState,
    true_range: WilderState,
    dx: WilderState,
    /// Latest `+DI`.
    pub plus_di: Option<Real>,
    /// Latest `-DI`.
    pub minus_di: Option<Real>,
    /// Latest `ADX`.
    pub adx: Option<Real>,
}

impl Adx {
    /// Create a new ADX over the given period.
    ///
    /// # Panics
    /// Panics if `period` is zero.
    pub fn new(period: usize) -> Self {
        Self {
            prev: None,
            plus_dm: WilderState::new(period),
            minus_dm: WilderState::new(period),
            true_range: WilderState::new(period),
            dx: WilderState::new(period),
            plus_di: None,
            minus_di: None,
            adx: None,
        }
    }
}

impl Indicator for Adx {
    type Input = Candle;
    type Output = AdxValue;

    fn update(&mut self, candle: Candle) -> Option<AdxValue> {
        let (prev_high, prev_low, prev_close) = match self.prev {
            Some(prev) => prev,
            None => {
                // First bar: no directional movement to measure yet.
                self.prev = Some((candle.high, candle.low, candle.close));
                return None;
            }
        };
        self.prev = Some((candle.high, candle.low, candle.close));

        let up_move = candle.high - prev_high;
        let down_move = prev_low - candle.low;
        let plus_dm = if up_move > down_move && up_move > 0.0 {
            up_move
        } else {
            0.0
        };
        let minus_dm = if down_move > up_move && down_move > 0.0 {
            down_move
        } else {
            0.0
        };
        let high_low = candle.high - candle.low;
        let high_close = (candle.high - prev_close).abs();
        let low_close = (candle.low - prev_close).abs();
        let tr = high_low.max(high_close).max(low_close);

        let smoothed_plus = self.plus_dm.update(plus_dm);
        let smoothed_minus = self.minus_dm.update(minus_dm);
        let smoothed_tr = self.true_range.update(tr);

        if let (Some(sp), Some(sm), Some(st)) = (smoothed_plus, smoothed_minus, smoothed_tr) {
            let (plus_di, minus_di) = if st == 0.0 {
                (0.0, 0.0)
            } else {
                (100.0 * sp / st, 100.0 * sm / st)
            };
            self.plus_di = Some(plus_di);
            self.minus_di = Some(minus_di);

            let sum = plus_di + minus_di;
            let dx = if sum == 0.0 {
                0.0
            } else {
                100.0 * (plus_di - minus_di).abs() / sum
            };
            self.adx = self.dx.update(dx);
        }

        self.current()
    }

    fn current(&self) -> Option<AdxValue> {
        match (self.plus_di, self.minus_di, self.adx) {
            (Some(plus_di), Some(minus_di), Some(adx)) => Some(AdxValue {
                plus_di,
                minus_di,
                adx,
            }),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.prev = None;
        self.plus_dm.reset();
        self.minus_dm.reset();
        self.true_range.reset();
        self.dx.reset();
        self.plus_di = None;
        self.minus_di = None;
        self.adx = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_uptrend_has_plus_di_above_minus_di() {
        let mut adx = Adx::new(3);
        let mut last = None;
        // Steadily rising bars: +DI should dominate -DI.
        for i in 0..12 {
            let base = 10.0 + i as Real;
            last = adx.update(Candle::new(base, base + 1.0, base - 0.5, base + 0.5, 0.0));
        }
        let out = last.expect("adx should be ready");
        assert!(out.plus_di > out.minus_di);
        assert!((0.0..=100.0).contains(&out.adx));
    }
}
