//! Internal scalar recurrence helpers shared by the smoothing indicators.
//!
//! These operate on a plain `Real` stream (no source, no `Indicator` impl) so
//! that source-wrapping indicators ([`Ema`](super::Ema), [`Rsi`](super::Rsi),
//! [`Macd`](super::Macd), [`Adx`](super::Adx), …) can embed one or more without
//! re-deriving the math.

use crate::types::Real;

/// EMA recurrence; seeds on the first sample, then
/// `ema = alpha * x + (1 - alpha) * prev`.
#[derive(Debug, Clone)]
pub(crate) struct EmaState {
    alpha: Real,
    pub value: Option<Real>,
}

impl EmaState {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "EMA period must be greater than zero");
        Self::with_alpha(2.0 / (period as Real + 1.0))
    }

    pub fn with_alpha(alpha: Real) -> Self {
        assert!(alpha > 0.0 && alpha <= 1.0, "alpha must be in (0, 1]");
        Self { alpha, value: None }
    }

    pub fn update(&mut self, input: Real) -> Option<Real> {
        let next = match self.value {
            Some(prev) => self.alpha * input + (1.0 - self.alpha) * prev,
            None => input,
        };
        self.value = Some(next);
        self.value
    }

    pub fn reset(&mut self) {
        self.value = None;
    }
}

/// Wilder smoothing (RMA / SMMA) recurrence; seeds with the mean of the first
/// `period` samples, then `rma = (prev * (period - 1) + x) / period`.
#[derive(Debug, Clone)]
pub(crate) struct WilderState {
    period: usize,
    seen: usize,
    sum: Real,
    pub value: Option<Real>,
}

impl WilderState {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be greater than zero");
        Self {
            period,
            seen: 0,
            sum: 0.0,
            value: None,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    pub fn update(&mut self, input: Real) -> Option<Real> {
        match self.value {
            Some(prev) => {
                let p = self.period as Real;
                self.value = Some((prev * (p - 1.0) + input) / p);
            }
            None => {
                self.seen += 1;
                self.sum += input;
                if self.seen == self.period {
                    self.value = Some(self.sum / self.period as Real);
                }
            }
        }
        self.value
    }

    pub fn reset(&mut self) {
        self.seen = 0;
        self.sum = 0.0;
        self.value = None;
    }
}
