//! Internal rolling-window statistics core shared by the windowed indicators.
//!
//! Maintains the last `period` samples plus running sum and sum-of-squares, so
//! `mean` and (population) `variance`/`stddev` are O(1) per update. Embedded by
//! [`Sma`](super::Sma), [`StdDev`](super::StdDev) and
//! [`Bollinger`](super::Bollinger) — anything needing a moving average and/or
//! dispersion over the same window.

use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::indicators::ops::ExtremeOp;
use crate::types::Real;

#[derive(Debug, Clone)]
pub(crate) struct WindowStats {
    period: usize,
    window: VecDeque<Real>,
    sum: Real,
    sum_sq: Real,
}

impl WindowStats {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "window period must be greater than zero");
        Self {
            period,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
            sum_sq: 0.0,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    /// Push a sample, evicting the oldest once the window is full. Returns
    /// whether the window is now full (i.e. statistics are valid).
    pub fn update(&mut self, x: Real) -> bool {
        self.window.push_back(x);
        self.sum += x;
        self.sum_sq += x * x;
        if self.window.len() > self.period {
            let old = self.window.pop_front().expect("window is non-empty");
            self.sum -= old;
            self.sum_sq -= old * old;
        }
        self.is_full()
    }

    pub fn is_full(&self) -> bool {
        self.window.len() == self.period
    }

    /// Mean over the window. Only meaningful once [`is_full`](Self::is_full).
    pub fn mean(&self) -> Real {
        self.sum / self.period as Real
    }

    /// Population variance over the window (clamped to non-negative against
    /// floating-point round-off).
    pub fn variance(&self) -> Real {
        let n = self.period as Real;
        let mean = self.sum / n;
        (self.sum_sq / n - mean * mean).max(0.0)
    }

    /// Population standard deviation over the window.
    pub fn stddev(&self) -> Real {
        self.variance().sqrt()
    }

    pub fn reset(&mut self) {
        self.window.clear();
        self.sum = 0.0;
        self.sum_sq = 0.0;
    }
}

/// Rolling extremum over the last `period` samples via a monotonic deque, so
/// each update is O(1) amortised. The direction (max/min) is the [`ExtremeOp`]
/// marker. Embedded by [`Extreme`](super::ops::Extreme) (→ `RollingMax`/
/// `RollingMin`) and by [`Stochastic`](super::Stochastic).
#[derive(Debug, Clone)]
pub(crate) struct WindowExtreme<Op> {
    period: usize,
    // (index, value), kept monotonic so the front is always the extremum.
    deque: VecDeque<(usize, Real)>,
    count: usize,
    _op: PhantomData<fn() -> Op>,
}

impl<Op> WindowExtreme<Op> {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "window period must be greater than zero");
        Self {
            period,
            deque: VecDeque::new(),
            count: 0,
            _op: PhantomData,
        }
    }

    pub fn period(&self) -> usize {
        self.period
    }

    pub fn reset(&mut self) {
        self.deque.clear();
        self.count = 0;
    }
}

impl<Op: ExtremeOp> WindowExtreme<Op> {
    /// Push a sample; returns the extremum over the window once `period` samples
    /// have been seen (`None` during warm-up).
    pub fn update(&mut self, x: Real) -> Option<Real> {
        let idx = self.count;
        self.count += 1;

        // Drop tail entries that `x` dominates: they can never be the extremum
        // while `x` is in the window.
        while let Some(&(_, back)) = self.deque.back() {
            if Op::dominates(x, back) {
                self.deque.pop_back();
            } else {
                break;
            }
        }
        self.deque.push_back((idx, x));

        // Drop the front once it has fallen out of the window.
        while let Some(&(front_idx, _)) = self.deque.front() {
            if front_idx + self.period <= idx {
                self.deque.pop_front();
            } else {
                break;
            }
        }

        if self.count >= self.period {
            Some(self.deque.front().expect("deque is non-empty").1)
        } else {
            None
        }
    }
}
