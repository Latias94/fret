#[cfg(any(
    feature = "state-mutation",
    feature = "state-query",
    feature = "state-selector"
))]
use std::any::Any;
#[cfg(any(feature = "state-mutation", feature = "state-query"))]
use std::future::Future;
#[cfg(feature = "state-mutation")]
use std::sync::Arc;

use fret_ui::ElementContext;
#[cfg(feature = "state-selector")]
use fret_ui::Invalidation;

use super::super::RenderContextAccess;
#[cfg(feature = "state-query")]
use super::query_snapshot_entry_for_key;
#[cfg(feature = "state-selector")]
use super::{LocalSelectorLayoutInputs, ModelSelectorInputs};
#[cfg(feature = "state-mutation")]
use super::{LocalStateTxn, take_mutation_completion_state_in, take_mutation_success_in};

/// Grouped selector/query helpers for extracted app-render helpers on the default app surface.
#[doc(hidden)]
pub struct AppRenderData<'cx, 'a> {
    #[allow(dead_code)]
    pub(super) cx: &'cx mut ElementContext<'a, crate::app::App>,
}

impl<'cx, 'a> AppRenderData<'cx, 'a> {
    /// Default LocalState-first selector path for extracted app-render helpers on the app-facing
    /// lane.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags on extracted app-render helpers
    /// when the derived value affects layout.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags on extracted app-render helpers
    /// when the derived value affects paint.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_paint<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, crate::app::App>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Paint),
            move |cx| compute(inputs.values_in(cx, Invalidation::Paint)),
        )
    }

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector<Deps, TValue>(
        self,
        deps: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, crate::app::App>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(self.cx, deps, compute)
    }

    #[cfg(feature = "state-query")]
    pub fn query<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Result<T, fret_query::QueryError>
        + Send
        + 'static,
    ) -> fret_query::QueryHandle<T> {
        fret_query::ui::QueryElementContextExt::use_query(self.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + Send + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + Send + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async(self.cx, key, policy, fetch)
    }

    #[cfg(feature = "state-query")]
    pub fn query_async_local<T, Fut>(
        self,
        key: fret_query::QueryKey<T>,
        policy: fret_query::QueryPolicy,
        fetch: impl FnOnce(fret_query::CancellationToken) -> Fut + 'static,
    ) -> fret_query::QueryHandle<T>
    where
        T: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<T, fret_query::QueryError>> + 'static,
    {
        fret_query::ui::QueryElementContextExt::use_query_async_local(self.cx, key, policy, fetch)
    }

    /// Grouped query-client snapshot read for extracted app-render diagnostics helpers.
    ///
    /// Keep raw `fret::query::with_query_client(...)` for pure app/driver code that does not have
    /// a grouped `cx.data()` surface.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot> {
        fret_query::with_query_client(self.cx.app, |client, _app| client.snapshot())
    }

    /// Find one typed query snapshot entry from an extracted app-render helper on the grouped app
    /// data lane.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
    ) -> Option<fret_query::QuerySnapshotEntry> {
        self.query_snapshot()
            .and_then(|snapshot| query_snapshot_entry_for_key(snapshot, key))
    }

    /// Cancel one inflight query task from an extracted app-render helper while keeping redraw
    /// ownership on the grouped app data lane.
    #[cfg(feature = "state-query")]
    pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.cancel_inflight(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + Send + Sync + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + Send + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async(self.cx, policy, submit)
    }

    #[cfg(feature = "state-mutation")]
    pub fn mutation_async_local<TIn, TOut, Fut>(
        self,
        policy: fret_mutation::MutationPolicy,
        submit: impl Fn(fret_mutation::CancellationToken, Arc<TIn>) -> Fut + 'static,
    ) -> fret_mutation::MutationHandle<TIn, TOut>
    where
        TIn: Any + Send + Sync + 'static,
        TOut: Any + Send + Sync + 'static,
        Fut: Future<Output = Result<TOut, fret_mutation::MutationError>> + 'static,
    {
        fret_mutation::ui::MutationElementContextExt::use_mutation_async_local(
            self.cx, policy, submit,
        )
    }

    /// Consume a mutation completion exactly once for one `(effect_key, handle)` pair inside an
    /// extracted app-render helper.
    ///
    /// Prefer `update_after_mutation_completion(...)` when this once-only gate immediately drives
    /// app-owned model updates.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_completion_state_in(self.cx, effect_key, handle).is_some()
    }

    /// Update ordinary app-owned `LocalState<T>` or shared models exactly once after a mutation
    /// reaches a fresh terminal completion inside an extracted app-render helper.
    #[cfg(feature = "state-mutation")]
    pub fn update_after_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        apply: impl FnOnce(
            &mut fret_runtime::ModelStore,
            fret_mutation::MutationState<TIn, TOut>,
        ) -> bool,
    ) -> bool {
        let Some(state) = take_mutation_completion_state_in(self.cx, effect_key, handle) else {
            return false;
        };
        let changed = apply(self.cx.app.models_mut(), state);
        if changed {
            self.cx.app.request_redraw(self.cx.window);
        }
        changed
    }

    /// Project a fresh terminal mutation completion into app-local state without exposing
    /// `ModelStore` in extracted app-render helpers.
    #[cfg(feature = "state-mutation")]
    pub fn update_locals_after_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        apply: impl for<'m> FnOnce(
            &mut LocalStateTxn<'m>,
            fret_mutation::MutationState<TIn, TOut>,
        ) -> bool,
    ) -> bool {
        let Some(state) = take_mutation_completion_state_in(self.cx, effect_key, handle) else {
            return false;
        };
        let mut tx = LocalStateTxn {
            models: self.cx.app.models_mut(),
        };
        let changed = apply(&mut tx, state);
        if changed {
            self.cx.app.request_redraw(self.cx.window);
        }
        changed
    }

    /// Consume a mutation success exactly once for one `(effect_key, handle)` pair inside an
    /// extracted app-render helper.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_success_in(self.cx, effect_key, handle)
    }

    /// Grouped invalidation helper for extracted app-render helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    /// Grouped namespace invalidation helper for extracted app-render helpers.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query_namespace(self, namespace: &'static str) {
        let _ = fret_query::with_query_client(self.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.app.request_redraw(self.cx.window);
    }

    /// Invalidate one query key exactly once after a mutation reports success inside an extracted
    /// app-render helper.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_after_mutation_success<
        TIn: 'static,
        TOut: 'static,
        T: Any + Send + Sync + 'static,
    >(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        key: fret_query::QueryKey<T>,
    ) -> bool {
        if !take_mutation_success_in(self.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.app.request_redraw(self.cx.window);
        true
    }

    /// Invalidate one query namespace exactly once after a mutation reports success inside an
    /// extracted app-render helper.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        namespace: &'static str,
    ) -> bool {
        if !take_mutation_success_in(self.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.app.request_redraw(self.cx.window);
        true
    }
}

/// Brings the grouped `data()` namespace to extracted app-render helper functions.
pub trait AppRenderDataExt<'a> {
    /// Discover selector/query helpers through `cx.data()` rather than naming the carrier type
    /// directly.
    fn data(&mut self) -> AppRenderData<'_, 'a>;
}

impl<'a, Cx> AppRenderDataExt<'a> for Cx
where
    Cx: RenderContextAccess<'a, crate::app::App>,
{
    fn data(&mut self) -> AppRenderData<'_, 'a> {
        AppRenderData {
            cx: self.elements(),
        }
    }
}
