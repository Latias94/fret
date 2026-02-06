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

/// A compact dependency signature for selector-style memoization.
///
/// This is intended to be used as the `Deps` type parameter for [`Selector`]. The `ui` feature
/// provides a [`DepsBuilder`](crate::ui::DepsBuilder) that records model/global tokens while also
/// registering the corresponding observations on the element context.
#[derive(Clone, Default)]
pub struct DepsSignature {
    tokens: TokenList,
    #[cfg(debug_assertions)]
    pub(crate) observed_tokens: u16,
}

impl fmt::Debug for DepsSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DepsSignature")
            .field("tokens", &self.tokens.as_slice())
            .finish()
    }
}

impl PartialEq for DepsSignature {
    fn eq(&self, other: &Self) -> bool {
        self.tokens == other.tokens
    }
}

impl Eq for DepsSignature {}

impl DepsSignature {
    pub fn is_empty(&self) -> bool {
        self.tokens.as_slice().is_empty()
    }

    pub fn tokens(&self) -> &[u64] {
        self.tokens.as_slice()
    }

    pub fn push_token(&mut self, token: u64) {
        self.tokens.push(token);
    }
}

const INLINE_TOKEN_CAP: usize = 16;

#[derive(Clone)]
enum TokenList {
    Inline {
        len: u8,
        tokens: [u64; INLINE_TOKEN_CAP],
    },
    Heap(Vec<u64>),
}

impl Default for TokenList {
    fn default() -> Self {
        Self::Inline {
            len: 0,
            tokens: [0; INLINE_TOKEN_CAP],
        }
    }
}

impl fmt::Debug for TokenList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl PartialEq for TokenList {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for TokenList {}

impl TokenList {
    fn push(&mut self, token: u64) {
        match self {
            TokenList::Inline { len, tokens } => {
                let len_usize = *len as usize;
                if len_usize < INLINE_TOKEN_CAP {
                    tokens[len_usize] = token;
                    *len = (len_usize + 1) as u8;
                    return;
                }

                let mut heap = Vec::with_capacity(INLINE_TOKEN_CAP.saturating_mul(2));
                heap.extend_from_slice(tokens.as_slice());
                heap.push(token);
                *self = TokenList::Heap(heap);
            }
            TokenList::Heap(tokens) => tokens.push(token),
        }
    }

    fn as_slice(&self) -> &[u64] {
        match self {
            TokenList::Inline { len, tokens } => &tokens[..(*len as usize)],
            TokenList::Heap(tokens) => tokens.as_slice(),
        }
    }
}

#[cfg(feature = "ui")]
pub mod ui;

#[cfg(test)]
mod tests {
    use super::DepsSignature;
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

    #[test]
    fn deps_signature_compares_by_tokens() {
        let mut a = DepsSignature::default();
        a.push_token(1);
        a.push_token(2);

        let mut b = DepsSignature::default();
        b.push_token(1);
        b.push_token(2);

        assert_eq!(a, b);

        b.push_token(3);
        assert_ne!(a, b);
    }
}
