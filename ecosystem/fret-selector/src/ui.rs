use std::any::Any;
use std::hash::Hash;
use std::panic::Location;

use fret_runtime::Model;
use fret_ui::{ElementContext, Invalidation, UiHost};

use crate::{DepsSignature, Selector};

const MISSING_TOKEN: u64 = u64::MAX;

struct SelectorMemoState<Deps, TValue> {
    selector: Selector<Deps, TValue>,
    #[cfg(debug_assertions)]
    last_frame_id: u64,
    #[cfg(debug_assertions)]
    calls_in_frame: u32,
    #[cfg(debug_assertions)]
    warned_unobserved_deps: bool,
}

impl<Deps, TValue> SelectorMemoState<Deps, TValue> {
    fn new() -> Self {
        Self {
            selector: Selector::new(),
            #[cfg(debug_assertions)]
            last_frame_id: 0,
            #[cfg(debug_assertions)]
            calls_in_frame: 0,
            #[cfg(debug_assertions)]
            warned_unobserved_deps: false,
        }
    }

    #[cfg(debug_assertions)]
    fn record_call(&mut self, frame_id: u64, callsite: (&'static str, u32, u32)) {
        if self.last_frame_id != frame_id {
            self.last_frame_id = frame_id;
            self.calls_in_frame = 0;
        }

        self.calls_in_frame = self.calls_in_frame.saturating_add(1);
        if self.calls_in_frame == 2 {
            tracing::warn!(
                file = callsite.0,
                line = callsite.1,
                column = callsite.2,
                "selector called multiple times per frame at the same callsite; wrap in `cx.keyed(...)` or use `use_selector_keyed(...)` to avoid state collisions"
            );
        }
    }

    #[cfg(debug_assertions)]
    fn maybe_warn_unobserved_deps(&mut self, deps: &dyn Any, callsite: (&'static str, u32, u32)) {
        if self.warned_unobserved_deps {
            return;
        }

        let Some(sig) = deps.downcast_ref::<DepsSignature>() else {
            return;
        };

        if !sig.is_empty() && sig.observed_tokens == 0 {
            tracing::warn!(
                file = callsite.0,
                line = callsite.1,
                column = callsite.2,
                "DepsSignature produced no observed tokens; build deps with `DepsBuilder` (or ensure deps closure observes its dependencies every frame)"
            );
            self.warned_unobserved_deps = true;
        }
    }
}

/// Helper for building selector dependency signatures from observed models/globals.
///
/// This avoids the common footgun where callers encode `Model`/global revisions in their deps
/// signature but forget to register the corresponding observations every frame.
pub struct DepsBuilder<'cx, 'a, H: UiHost> {
    cx: &'cx mut ElementContext<'a, H>,
    deps: DepsSignature,
}

impl<'cx, 'a, H: UiHost> DepsBuilder<'cx, 'a, H> {
    pub fn new(cx: &'cx mut ElementContext<'a, H>) -> Self {
        Self {
            cx,
            deps: DepsSignature::default(),
        }
    }

    pub fn model_rev<T: Any>(&mut self, model: &Model<T>) -> &mut Self {
        self.model_rev_invalidation(model, Invalidation::Paint)
    }

    pub fn model_rev_invalidation<T: Any>(
        &mut self,
        model: &Model<T>,
        invalidation: Invalidation,
    ) -> &mut Self {
        let rev = observed_model_revision(self.cx, model, invalidation);
        self.deps.push_token(rev.unwrap_or(MISSING_TOKEN));
        #[cfg(debug_assertions)]
        {
            self.deps.observed_tokens = self.deps.observed_tokens.saturating_add(1);
        }
        self
    }

    pub fn global_token<T: Any>(&mut self) -> &mut Self {
        self.global_token_invalidation::<T>(Invalidation::Paint)
    }

    pub fn global_token_invalidation<T: Any>(&mut self, invalidation: Invalidation) -> &mut Self {
        let token = observed_global_token::<T, H>(self.cx, invalidation);
        self.deps.push_token(token.unwrap_or(MISSING_TOKEN));
        #[cfg(debug_assertions)]
        {
            self.deps.observed_tokens = self.deps.observed_tokens.saturating_add(1);
        }
        self
    }

    pub fn token(&mut self, token: u64) -> &mut Self {
        self.deps.push_token(token);
        self
    }

    pub fn finish(self) -> DepsSignature {
        self.deps
    }
}

/// UI sugar for memoized derived state.
///
/// Important: the `deps` closure must **observe** the dependencies it encodes (models/globals)
/// every frame. The selector only decides whether to recompute the expensive `compute` closure.
pub trait SelectorElementContextExt {
    #[track_caller]
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;

    #[track_caller]
    fn use_selector_keyed<K: Hash, Deps, TValue>(
        &mut self,
        key: K,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;

    #[doc(hidden)]
    fn use_selector_at<Deps, TValue>(
        &mut self,
        callsite: &'static Location<'static>,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;

    #[doc(hidden)]
    fn use_selector_keyed_at<K: Hash, Deps, TValue>(
        &mut self,
        callsite: &'static Location<'static>,
        key: K,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone;
}

impl<'a, H: UiHost> SelectorElementContextExt for ElementContext<'a, H> {
    #[track_caller]
    fn use_selector<Deps, TValue>(
        &mut self,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let callsite = Location::caller();
        let key = (callsite.file(), callsite.line(), callsite.column());
        let frame_id = self.frame_id.0;
        let callsite_key = key;

        self.keyed(key, |cx| {
            let deps_value = deps(cx);

            let cached = cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                #[cfg(debug_assertions)]
                state.record_call(frame_id, callsite_key);
                #[cfg(debug_assertions)]
                state.maybe_warn_unobserved_deps(&deps_value as &dyn Any, callsite_key);

                state.selector.get_if_deps(&deps_value).cloned()
            });

            if let Some(value) = cached {
                return value;
            }

            let value = compute(cx);
            cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                state.selector.set(deps_value, value.clone());
            });
            value
        })
    }

    fn use_selector_at<Deps, TValue>(
        &mut self,
        callsite: &'static Location<'static>,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let key = (callsite.file(), callsite.line(), callsite.column());
        let frame_id = self.frame_id.0;
        let callsite_key = key;

        self.keyed(key, |cx| {
            let deps_value = deps(cx);

            let cached = cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                #[cfg(debug_assertions)]
                state.record_call(frame_id, callsite_key);
                #[cfg(debug_assertions)]
                state.maybe_warn_unobserved_deps(&deps_value as &dyn Any, callsite_key);

                state.selector.get_if_deps(&deps_value).cloned()
            });

            if let Some(value) = cached {
                return value;
            }

            let value = compute(cx);
            cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                state.selector.set(deps_value, value.clone());
            });
            value
        })
    }

    fn use_selector_keyed<K: Hash, Deps, TValue>(
        &mut self,
        key: K,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let callsite = Location::caller();
        let callsite_key = (callsite.file(), callsite.line(), callsite.column());
        let frame_id = self.frame_id.0;

        self.keyed((callsite_key, key), |cx| {
            let deps_value = deps(cx);

            let cached = cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                #[cfg(debug_assertions)]
                state.record_call(frame_id, callsite_key);
                #[cfg(debug_assertions)]
                state.maybe_warn_unobserved_deps(&deps_value as &dyn Any, callsite_key);

                state.selector.get_if_deps(&deps_value).cloned()
            });

            if let Some(value) = cached {
                return value;
            }

            let value = compute(cx);
            cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                state.selector.set(deps_value, value.clone());
            });
            value
        })
    }

    fn use_selector_keyed_at<K: Hash, Deps, TValue>(
        &mut self,
        callsite: &'static Location<'static>,
        key: K,
        deps: impl FnOnce(&mut Self) -> Deps,
        compute: impl FnOnce(&mut Self) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        let callsite_key = (callsite.file(), callsite.line(), callsite.column());
        let frame_id = self.frame_id.0;
        self.keyed((callsite_key, key), |cx| {
            let deps_value = deps(cx);

            let cached = cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                #[cfg(debug_assertions)]
                state.record_call(frame_id, callsite_key);
                #[cfg(debug_assertions)]
                state.maybe_warn_unobserved_deps(&deps_value as &dyn Any, callsite_key);

                state.selector.get_if_deps(&deps_value).cloned()
            });

            if let Some(value) = cached {
                return value;
            }

            let value = compute(cx);
            cx.with_state(SelectorMemoState::<Deps, TValue>::new, |state| {
                state.selector.set(deps_value, value.clone());
            });
            value
        })
    }
}

pub fn observed_model_revision<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: &Model<T>,
    invalidation: Invalidation,
) -> Option<u64> {
    cx.observe_model(model, invalidation);
    cx.app.models().revision(model)
}

pub fn observed_global_token<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Option<u64> {
    cx.observe_global::<T>(invalidation);
    cx.app.global_revision_of::<T>()
}
