//! Core scalar and market-data types shared across the crate.

/// The scalar type used throughout the crate for prices and indicator outputs.
///
/// Centralised as an alias so the whole library can be switched to another
/// floating-point width (or a fixed-point type) in one place.
pub type Real = f64;

/// A single OHLCV bar.
///
/// Indicators that only need a price stream take [`Real`] directly; those that
/// need the full bar (true range, typical price, volume-weighted values, …)
/// take a `Candle` as their [`Indicator::Input`](crate::Indicator::Input).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub open: Real,
    pub high: Real,
    pub low: Real,
    pub close: Real,
    pub volume: Real,
}

impl Candle {
    pub fn new(open: Real, high: Real, low: Real, close: Real, volume: Real) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Typical price: `(high + low + close) / 3`.
    pub fn typical(&self) -> Real {
        (self.high + self.low + self.close) / 3.0
    }

    /// Median price: `(high + low) / 2`.
    pub fn median(&self) -> Real {
        (self.high + self.low) / 2.0
    }
}
