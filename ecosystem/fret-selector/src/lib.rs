//! Selector-style derived state helpers (ecosystem-level).
//!
//! This crate provides a small, explicit "derived state" primitive:
//! - **memoize** an expensive computation behind a dependency signature (`Deps: PartialEq`)
//! - keep the kernel (`fret-ui`) mechanism-only (ADR 0066)
//!
//! A selector is intentionally *not* a global reactive graph. It is a local cache that can live in:
//! - element state (`ElementContext::with_state_*`) via the optional `ui` feature, or
//! - app/window state (e.g. inside a `Model<T>` payload)
//!
//! Design constraints:
//! - no hidden `ModelStore` borrows across user code (user code runs outside of any store guard)
//! - dependency tracking is explicit: callers must provide a dependency signature

use std::fmt;

#[derive(Default)]
pub struct Selector<Deps, TValue> {
    deps: Option<Deps>,
    value: Option<TValue>,
}

impl<Deps, TValue> fmt::Debug for Selector<Deps, TValue>
where
    Deps: fmt::Debug,
    TValue: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Selector")
            .field("deps", &self.deps)
            .field("value", &self.value)
            .finish()
    }
}

impl<Deps, TValue> Selector<Deps, TValue> {
    pub fn new() -> Self {
        Self {
            deps: None,
            value: None,
        }
    }

    pub fn invalidate(&mut self) {
        self.deps = None;
        self.value = None;
    }

    pub fn deps(&self) -> Option<&Deps> {
        self.deps.as_ref()
    }

    pub fn value(&self) -> Option<&TValue> {
        self.value.as_ref()
    }

    pub fn set(&mut self, deps: Deps, value: TValue) {
        self.deps = Some(deps);
        self.value = Some(value);
    }
}

impl<Deps: PartialEq, TValue> Selector<Deps, TValue> {
    pub fn get_if_deps(&self, deps: &Deps) -> Option<&TValue> {
        match self.deps.as_ref() {
            Some(prev) if prev == deps => self.value.as_ref(),
            _ => None,
        }
    }

    pub fn compute_ref(&mut self, deps: Deps, compute: impl FnOnce() -> TValue) -> &TValue {
        let should_recompute = match self.deps.as_ref() {
            Some(prev) if prev == &deps => self.value.is_none(),
            _ => true,
        };

        if should_recompute {
            self.set(deps, compute());
        }

        self.value.as_ref().expect("selector missing value")
    }

    pub fn compute(&mut self, deps: Deps, compute: impl FnOnce() -> TValue) -> TValue
    where
        TValue: Clone,
    {
        self.compute_ref(deps, compute).clone()
    }
}

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(test)]
mod tests {
    use super::Selector;

    #[test]
    fn selector_memoizes_by_deps() {
        let mut selector = Selector::<u32, String>::new();
        let mut calls = 0usize;

        let a = selector.compute(1, || {
            calls += 1;
            "a".to_string()
        });
        assert_eq!(a, "a");
        assert_eq!(calls, 1);

        let cached = selector.compute(1, || {
            calls += 1;
            "b".to_string()
        });
        assert_eq!(cached, "a");
        assert_eq!(calls, 1);

        let c = selector.compute(2, || {
            calls += 1;
            "c".to_string()
        });
        assert_eq!(c, "c");
        assert_eq!(calls, 2);
    }

    #[test]
    fn selector_invalidate_clears_cache() {
        let mut selector = Selector::<u32, String>::new();
        let a = selector.compute(1, || "a".to_string());
        assert_eq!(a, "a");

        selector.invalidate();

        let b = selector.compute(1, || "b".to_string());
        assert_eq!(b, "b");
    }
}
