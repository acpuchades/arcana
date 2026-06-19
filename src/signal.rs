//! The core [`Signal`] trait and its composition combinators.

/// An incremental, composable boolean condition.
///
/// Like an [`Indicator`](crate::Indicator), a signal carries its own state and
/// is advanced via [`update`](Signal::update). It yields a `bool` each step and
/// exposes the latest state through [`value`](Signal::value). Signals over the
/// same input type can be combined with the [`SignalExt`] combinators.
pub trait Signal {
    /// The per-sample input.
    type Input;

    /// Feed the next sample and return the current boolean state.
    fn update(&mut self, input: Self::Input) -> bool;

    /// The most recent boolean state, without advancing.
    fn value(&self) -> bool;

    /// Reset all internal state.
    fn reset(&mut self);
}

/// Combinator methods for building compound signals.
///
/// Implemented for every [`Signal`], so `a.and(b)`, `a.or(b).not()`,
/// `s.changed()` etc. work out of the box. Binary combinators feed the *same*
/// input to both sides, which is why they require `Self::Input: Clone`.
pub trait SignalExt: Signal + Sized {
    /// Logical AND of `self` and `rhs`.
    fn and<R>(self, rhs: R) -> And<Self, R>
    where
        R: Signal<Input = Self::Input>,
        Self::Input: Clone,
    {
        And {
            lhs: self,
            rhs,
            value: false,
        }
    }

    /// Logical OR of `self` and `rhs`.
    fn or<R>(self, rhs: R) -> Or<Self, R>
    where
        R: Signal<Input = Self::Input>,
        Self::Input: Clone,
    {
        Or {
            lhs: self,
            rhs,
            value: false,
        }
    }

    /// Logical XOR of `self` and `rhs`.
    fn xor<R>(self, rhs: R) -> Xor<Self, R>
    where
        R: Signal<Input = Self::Input>,
        Self::Input: Clone,
    {
        Xor {
            lhs: self,
            rhs,
            value: false,
        }
    }

    /// Logical negation of `self`.
    fn not(self) -> Not<Self> {
        Not {
            inner: self,
            value: false,
        }
    }

    /// Fires on the single step where `self`'s value toggles (in either
    /// direction).
    ///
    /// This is the one edge primitive. Directional events compose from it:
    /// "became true" is `s.changed().and(s)` and a crossover is
    /// `a.gt(b).and(a.gt(b).changed())` — see
    /// [`crosses_above`](crate::signals::IndicatorExt::crosses_above).
    fn changed(self) -> Change<Self> {
        Change {
            inner: self,
            prev: None,
            value: false,
        }
    }
}

impl<S: Signal> SignalExt for S {}

/// Logical AND of two signals. Created via [`SignalExt::and`].
#[derive(Debug, Clone)]
pub struct And<L, R> {
    lhs: L,
    rhs: R,
    value: bool,
}

impl<L, R> Signal for And<L, R>
where
    L: Signal,
    R: Signal<Input = L::Input>,
    L::Input: Clone,
{
    type Input = L::Input;

    fn update(&mut self, input: Self::Input) -> bool {
        let lhs = self.lhs.update(input.clone());
        let rhs = self.rhs.update(input);
        self.value = lhs && rhs;
        self.value
    }

    fn value(&self) -> bool {
        self.value
    }

    fn reset(&mut self) {
        self.lhs.reset();
        self.rhs.reset();
        self.value = false;
    }
}

/// Logical OR of two signals. Created via [`SignalExt::or`].
#[derive(Debug, Clone)]
pub struct Or<L, R> {
    lhs: L,
    rhs: R,
    value: bool,
}

impl<L, R> Signal for Or<L, R>
where
    L: Signal,
    R: Signal<Input = L::Input>,
    L::Input: Clone,
{
    type Input = L::Input;

    fn update(&mut self, input: Self::Input) -> bool {
        let lhs = self.lhs.update(input.clone());
        let rhs = self.rhs.update(input);
        self.value = lhs || rhs;
        self.value
    }

    fn value(&self) -> bool {
        self.value
    }

    fn reset(&mut self) {
        self.lhs.reset();
        self.rhs.reset();
        self.value = false;
    }
}

/// Logical XOR of two signals. Created via [`SignalExt::xor`].
#[derive(Debug, Clone)]
pub struct Xor<L, R> {
    lhs: L,
    rhs: R,
    value: bool,
}

impl<L, R> Signal for Xor<L, R>
where
    L: Signal,
    R: Signal<Input = L::Input>,
    L::Input: Clone,
{
    type Input = L::Input;

    fn update(&mut self, input: Self::Input) -> bool {
        let lhs = self.lhs.update(input.clone());
        let rhs = self.rhs.update(input);
        self.value = lhs ^ rhs;
        self.value
    }

    fn value(&self) -> bool {
        self.value
    }

    fn reset(&mut self) {
        self.lhs.reset();
        self.rhs.reset();
        self.value = false;
    }
}

/// Logical negation of a signal. Created via [`SignalExt::not`].
#[derive(Debug, Clone)]
pub struct Not<S> {
    inner: S,
    value: bool,
}

impl<S: Signal> Signal for Not<S> {
    type Input = S::Input;

    fn update(&mut self, input: Self::Input) -> bool {
        self.value = !self.inner.update(input);
        self.value
    }

    fn value(&self) -> bool {
        self.value
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.value = false;
    }
}

/// Toggle (change) detector. Created via [`SignalExt::changed`].
///
/// Fires on the single step where the inner signal's value differs from the
/// previous step, in either direction. The first step never fires (no prior
/// value to compare against).
#[derive(Debug, Clone)]
pub struct Change<S> {
    inner: S,
    prev: Option<bool>,
    value: bool,
}

impl<S: Signal> Signal for Change<S> {
    type Input = S::Input;

    fn update(&mut self, input: Self::Input) -> bool {
        let now = self.inner.update(input);
        self.value = match self.prev {
            Some(prev) => now != prev,
            None => false,
        };
        self.prev = Some(now);
        self.value
    }

    fn value(&self) -> bool {
        self.value
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.prev = None;
        self.value = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Emits a scripted sequence of booleans, for exercising combinators.
    struct Script {
        values: Vec<bool>,
        idx: usize,
    }
    impl Script {
        fn new(values: &[bool]) -> Self {
            Self {
                values: values.to_vec(),
                idx: 0,
            }
        }
    }
    impl Signal for Script {
        type Input = ();
        fn update(&mut self, _: ()) -> bool {
            let v = self.values[self.idx];
            self.idx += 1;
            v
        }
        fn value(&self) -> bool {
            self.values[self.idx.saturating_sub(1)]
        }
        fn reset(&mut self) {
            self.idx = 0;
        }
    }

    #[test]
    fn boolean_combinators() {
        struct Const(bool);
        impl Signal for Const {
            type Input = ();
            fn update(&mut self, _: ()) -> bool {
                self.0
            }
            fn value(&self) -> bool {
                self.0
            }
            fn reset(&mut self) {}
        }
        assert!(Const(true).and(Const(true)).update(()));
        assert!(!Const(true).and(Const(false)).update(()));
        assert!(Const(false).or(Const(true)).update(()));
        assert!(Const(true).xor(Const(false)).update(()));
        assert!(Const(false).not().update(()));
    }

    #[test]
    fn change_fires_on_each_toggle() {
        let mut c = Script::new(&[false, false, true, true, false]).changed();
        assert!(!c.update(())); // first step: no prior
        assert!(!c.update(())); // false -> false
        assert!(c.update(())); // false -> true
        assert!(!c.update(())); // true -> true
        assert!(c.update(())); // true -> false
    }
}
