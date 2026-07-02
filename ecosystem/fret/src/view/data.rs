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

#[cfg(feature = "state-selector")]
use fret_runtime::Model;
#[cfg(any(feature = "state-mutation", feature = "state-selector"))]
use fret_ui::Invalidation;
use fret_ui::{ElementContext, UiHost};

#[cfg(feature = "state-selector")]
use super::LocalState;
#[cfg(feature = "state-mutation")]
use super::LocalStateTxn;
#[cfg(any(
    feature = "state-mutation",
    feature = "state-query",
    feature = "state-selector"
))]
use super::TrackedStateExt;
use super::{AppUi, RenderContextAccess};

/// App-facing layout-phase convenience reads for query handles on the default `fret` lane.
///
/// This intentionally collapses only the repeated `layout(...).value_or_default()` fallback for the
/// ordinary app path. Query creation (`key`, `policy`, `fetch`) and lifecycle branching
/// (`status` / `data` / `error`) stay explicit.
#[cfg(feature = "state-query")]
pub trait QueryHandleReadLayoutExt<T: 'static> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> fret_query::QueryState<T>
    where
        Cx: RenderContextAccess<'a, H>;
}

#[cfg(feature = "state-query")]
impl<T: 'static> QueryHandleReadLayoutExt<T> for fret_query::QueryHandle<T> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(&self, cx: &mut Cx) -> fret_query::QueryState<T>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        TrackedStateExt::layout(self, cx).value_or_default()
    }
}

/// App-facing layout-phase convenience reads for mutation handles on the default `fret` lane.
#[cfg(feature = "state-mutation")]
pub trait MutationHandleReadLayoutExt<TIn: 'static, TOut: 'static> {
    fn read_layout<'a, H: UiHost + 'a, Cx>(
        &self,
        cx: &mut Cx,
    ) -> fret_mutation::MutationState<TIn, TOut>
    where
        Cx: RenderContextAccess<'a, H>;
}

#[cfg(feature = "state-mutation")]
impl<TIn: 'static, TOut: 'static> MutationHandleReadLayoutExt<TIn, TOut>
    for fret_mutation::MutationHandle<TIn, TOut>
{
    fn read_layout<'a, H: UiHost + 'a, Cx>(
        &self,
        cx: &mut Cx,
    ) -> fret_mutation::MutationState<TIn, TOut>
    where
        Cx: RenderContextAccess<'a, H>,
    {
        TrackedStateExt::layout(self, cx).value_or_default()
    }
}

/// LocalState-aware selector dependency helpers for the explicit `fret::selector` lane.
///
/// This keeps `fret-selector` portable while still letting LocalState-first app code build
/// dependency signatures without bouncing through `clone_model()` or `local.model()`.
#[cfg(feature = "state-selector")]
pub(crate) trait LocalSelectorDepsBuilderExt {
    fn local_rev_invalidation<T: Any>(
        &mut self,
        local: &LocalState<T>,
        invalidation: Invalidation,
    ) -> &mut Self;
}

#[cfg(feature = "state-selector")]
impl<'cx, 'a, H: UiHost> LocalSelectorDepsBuilderExt
    for fret_selector::ui::DepsBuilder<'cx, 'a, H>
{
    fn local_rev_invalidation<T: Any>(
        &mut self,
        local: &LocalState<T>,
        invalidation: Invalidation,
    ) -> &mut Self {
        self.model_rev_invalidation(local.model(), invalidation)
    }
}

#[cfg(feature = "state-selector")]
fn local_selector_value_in<T: Any + Clone, H: UiHost>(
    local: &LocalState<T>,
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> T {
    let value = match invalidation {
        Invalidation::Paint => local.paint_in(cx).value(),
        Invalidation::Layout => local.layout_in(cx).value(),
        Invalidation::HitTest | Invalidation::HitTestOnly => local.hit_test_in(cx).value(),
    };
    value.expect("LocalState-first selector inputs should always resolve a tracked value")
}

#[cfg(feature = "state-selector")]
fn model_selector_value_in<T: Any + Clone, H: UiHost>(
    model: &Model<T>,
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> T {
    let value = match invalidation {
        Invalidation::Paint => cx.get_model_cloned(model, Invalidation::Paint),
        Invalidation::Layout => cx.get_model_cloned(model, Invalidation::Layout),
        Invalidation::HitTest | Invalidation::HitTestOnly => {
            cx.get_model_cloned(model, Invalidation::HitTest)
        }
    };
    value.expect("Model selector inputs should always resolve a tracked value")
}

/// App-facing LocalState selector inputs for the grouped `cx.data()` lane.
///
/// This trait is intentionally hidden from docs because app authors should use the methods on
/// `cx.data()` rather than naming the trait directly.
#[cfg(feature = "state-selector")]
#[doc(hidden)]
pub trait LocalSelectorLayoutInputs<'a, H: UiHost>: Copy {
    type Values;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature;

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values;
}

#[cfg(feature = "state-selector")]
impl<'a, H: UiHost, T: Any + Clone> LocalSelectorLayoutInputs<'a, H> for &LocalState<T> {
    type Values = T;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature {
        let mut deps = fret_selector::ui::DepsBuilder::new(cx);
        deps.local_rev_invalidation(self, invalidation);
        deps.finish()
    }

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values {
        local_selector_value_in(self, cx, invalidation)
    }
}

#[cfg(feature = "state-selector")]
macro_rules! impl_local_selector_inputs_tuple {
    ($(($($name:ident:$idx:tt),+)),+ $(,)?) => {
        $(
            impl<'a, H: UiHost, $($name: Any + Clone),+> LocalSelectorLayoutInputs<'a, H>
                for ($(&LocalState<$name>,)+)
            {
                type Values = ($($name,)+);

                fn deps_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> fret_selector::DepsSignature {
                    let mut deps = fret_selector::ui::DepsBuilder::new(cx);
                    $(deps.local_rev_invalidation(self.$idx, invalidation);)+
                    deps.finish()
                }

                fn values_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> Self::Values {
                    ($(local_selector_value_in(self.$idx, cx, invalidation),)+)
                }
            }
        )+
    };
}

#[cfg(feature = "state-selector")]
impl_local_selector_inputs_tuple!(
    (A:0, B:1),
    (A:0, B:1, C:2),
    (A:0, B:1, C:2, D:3),
    (A:0, B:1, C:2, D:3, E:4),
    (A:0, B:1, C:2, D:3, E:4, F:5),
);

/// Explicit shared-`Model<T>` selector inputs for grouped derived reads on the app-facing lane.
///
/// This keeps the advanced/manual path out of raw `selector(deps, compute)` boilerplate when the
/// state already lives in an explicit `Model<T>` bag instead of `LocalState<T>`.
#[cfg(feature = "state-selector")]
#[doc(hidden)]
pub trait ModelSelectorInputs<'a, H: UiHost>: Copy {
    type Values;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature;

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values;
}

#[cfg(feature = "state-selector")]
impl<'a, H: UiHost, T: Any + Clone> ModelSelectorInputs<'a, H> for &Model<T> {
    type Values = T;

    fn deps_in(
        self,
        cx: &mut ElementContext<'a, H>,
        invalidation: Invalidation,
    ) -> fret_selector::DepsSignature {
        let mut deps = fret_selector::ui::DepsBuilder::new(cx);
        deps.model_rev_invalidation(self, invalidation);
        deps.finish()
    }

    fn values_in(self, cx: &mut ElementContext<'a, H>, invalidation: Invalidation) -> Self::Values {
        model_selector_value_in(self, cx, invalidation)
    }
}

#[cfg(feature = "state-selector")]
macro_rules! impl_model_selector_inputs_tuple {
    ($(($($name:ident:$idx:tt),+)),+ $(,)?) => {
        $(
            impl<'a, H: UiHost, $($name: Any + Clone),+> ModelSelectorInputs<'a, H>
                for ($(&Model<$name>,)+)
            {
                type Values = ($($name,)+);

                fn deps_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> fret_selector::DepsSignature {
                    let mut deps = fret_selector::ui::DepsBuilder::new(cx);
                    $(deps.model_rev_invalidation(self.$idx, invalidation);)+
                    deps.finish()
                }

                fn values_in(
                    self,
                    cx: &mut ElementContext<'a, H>,
                    invalidation: Invalidation,
                ) -> Self::Values {
                    ($(model_selector_value_in(self.$idx, cx, invalidation),)+)
                }
            }
        )+
    };
}

#[cfg(feature = "state-selector")]
impl_model_selector_inputs_tuple!(
    (A:0, B:1),
    (A:0, B:1, C:2),
    (A:0, B:1, C:2, D:3),
    (A:0, B:1, C:2, D:3, E:4),
    (A:0, B:1, C:2, D:3, E:4, F:5),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10, L:11),
    (A:0, B:1, C:2, D:3, E:4, F:5, G:6, Hh:7, I:8, J:9, K:10, L:11, M:12),
);

/// Grouped selector/query helpers for the default app authoring surface.
#[doc(hidden)]
pub struct AppUiData<'view, 'cx, 'a, H: UiHost> {
    #[allow(dead_code)]
    pub(super) cx: &'view mut AppUi<'cx, 'a, H>,
}

#[cfg(feature = "state-mutation")]
fn take_mutation_completion_state_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> Option<fret_mutation::MutationState<TIn, TOut>> {
    let completion_token = handle.completion_token();
    let state = cx
        .get_model_cloned(handle.model(), Invalidation::Layout)
        .unwrap_or_default();
    if !(state.is_success() || state.is_error()) {
        return None;
    }

    let Some(completion_token) = completion_token else {
        return None;
    };

    let fresh = cx.keyed_slot_state(
        (effect_key, handle.model().id()),
        Option::<std::num::NonZeroU64>::default,
        |last_seen| {
            if *last_seen == Some(completion_token) {
                false
            } else {
                *last_seen = Some(completion_token);
                true
            }
        },
    );
    if !fresh {
        return None;
    }

    Some(state)
}

#[cfg(feature = "state-mutation")]
fn take_mutation_completion_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> bool {
    take_mutation_completion_state_in(cx, effect_key, handle).is_some()
}

#[cfg(feature = "state-mutation")]
fn take_mutation_success_in<H: UiHost, TIn: 'static, TOut: 'static>(
    cx: &mut ElementContext<'_, H>,
    effect_key: u64,
    handle: &fret_mutation::MutationHandle<TIn, TOut>,
) -> bool {
    let success_token = handle.success_token();
    let state = cx
        .get_model_cloned(handle.model(), Invalidation::Layout)
        .unwrap_or_default();
    if !state.is_success() {
        return false;
    }

    let Some(success_token) = success_token else {
        return false;
    };

    cx.keyed_slot_state(
        (effect_key, handle.model().id()),
        Option::<std::num::NonZeroU64>::default,
        |last_seen| {
            if *last_seen == Some(success_token) {
                false
            } else {
                *last_seen = Some(success_token);
                true
            }
        },
    )
}

#[cfg(feature = "state-query")]
fn query_snapshot_entry_for_key<T: Any + Send + Sync + 'static>(
    snapshot: fret_query::QueryClientSnapshot,
    key: fret_query::QueryKey<T>,
) -> Option<fret_query::QuerySnapshotEntry> {
    let type_name = std::any::type_name::<T>();
    snapshot.entries.into_iter().find(|entry| {
        entry.namespace == key.namespace()
            && entry.hash == key.hash()
            && entry.type_name == type_name
    })
}

impl<'view, 'cx, 'a, H: UiHost> AppUiData<'view, 'cx, 'a, H> {
    /// Default LocalState-first selector path for app-facing derived values that affect layout.
    ///
    /// Use this when the deps are view-owned `LocalState<T>` slots. Keep raw `selector(...)` for
    /// explicit shared `Model<T>` signatures, global tokens, or custom dependency builders.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: LocalSelectorLayoutInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags that affect layout.
    ///
    /// Use this when the deps intentionally stay as shared `Model<T>` handles on manual/advanced
    /// surfaces. Prefer `selector_layout(...)` when the inputs are view-owned `LocalState<T>`.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_layout<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Layout),
            move |cx| compute(inputs.values_in(cx, Invalidation::Layout)),
        )
    }

    /// Grouped selector path for explicit shared `Model<T>` bags that affect paint-time derived
    /// values.
    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector_model_paint<Inputs, TValue>(
        self,
        inputs: Inputs,
        compute: impl FnOnce(Inputs::Values) -> TValue,
    ) -> TValue
    where
        Inputs: ModelSelectorInputs<'a, H>,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(
            self.cx.cx,
            move |cx| inputs.deps_in(cx, Invalidation::Paint),
            move |cx| compute(inputs.values_in(cx, Invalidation::Paint)),
        )
    }

    #[track_caller]
    #[cfg(feature = "state-selector")]
    pub fn selector<Deps, TValue>(
        self,
        deps: impl FnOnce(&mut ElementContext<'a, H>) -> Deps,
        compute: impl FnOnce(&mut ElementContext<'a, H>) -> TValue,
    ) -> TValue
    where
        Deps: Any + PartialEq,
        TValue: Any + Clone,
    {
        fret_selector::ui::SelectorElementContextExt::use_selector(self.cx.cx, deps, compute)
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
        fret_query::ui::QueryElementContextExt::use_query(self.cx.cx, key, policy, fetch)
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
        fret_query::ui::QueryElementContextExt::use_query_async(self.cx.cx, key, policy, fetch)
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
        fret_query::ui::QueryElementContextExt::use_query_async_local(
            self.cx.cx, key, policy, fetch,
        )
    }

    /// Grouped query-client snapshot read for app-facing diagnostics or status chrome on `AppUi`.
    ///
    /// Keep raw `fret::query::with_query_client(...)` for pure app/driver code that does not have
    /// a grouped `cx.data()` surface.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot(self) -> Option<fret_query::QueryClientSnapshot> {
        fret_query::with_query_client(self.cx.cx.app, |client, _app| client.snapshot())
    }

    /// Find one typed query snapshot entry on the grouped app data lane without dropping back to
    /// raw query-client plumbing.
    #[cfg(feature = "state-query")]
    pub fn query_snapshot_entry<T: Any + Send + Sync + 'static>(
        self,
        key: fret_query::QueryKey<T>,
    ) -> Option<fret_query::QuerySnapshotEntry> {
        self.query_snapshot()
            .and_then(|snapshot| query_snapshot_entry_for_key(snapshot, key))
    }

    /// Cancel one inflight query task from the grouped app data lane while keeping redraw ownership
    /// local to `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn cancel_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.cancel_inflight(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
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
        fret_mutation::ui::MutationElementContextExt::use_mutation_async(self.cx.cx, policy, submit)
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
            self.cx.cx, policy, submit,
        )
    }

    /// Consume a mutation completion exactly once for one `(effect_key, handle)` pair on the
    /// default app data lane.
    ///
    /// This covers both success and error terminal states when app code needs to materialize the
    /// latest terminal result into ordinary `LocalState<T>` or shared models without replaying the
    /// same completion on later renders. Prefer `update_after_mutation_completion(...)` when this
    /// once-only gate immediately drives app-owned model updates.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_completion<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_completion_in(self.cx.cx, effect_key, handle)
    }

    /// Update ordinary app-owned `LocalState<T>` or shared models exactly once after a mutation
    /// reaches a fresh terminal completion on the default app data lane.
    ///
    /// Prefer this over pairing `handle.read_layout(cx)` with `take_mutation_completion(...)`
    /// when the only goal is to project the latest terminal result into other app-owned state.
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
        let Some(state) = take_mutation_completion_state_in(self.cx.cx, effect_key, handle) else {
            return false;
        };
        let changed = apply(self.cx.cx.app.models_mut(), state);
        if changed {
            self.cx.cx.app.request_redraw(self.cx.cx.window);
        }
        changed
    }

    /// Project a fresh terminal mutation completion into app-local state without exposing
    /// `ModelStore` in public app code.
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
        let Some(state) = take_mutation_completion_state_in(self.cx.cx, effect_key, handle) else {
            return false;
        };
        let mut tx = LocalStateTxn {
            models: self.cx.cx.app.models_mut(),
        };
        let changed = apply(&mut tx, state);
        if changed {
            self.cx.cx.app.request_redraw(self.cx.cx.window);
        }
        changed
    }

    /// Consume a mutation success exactly once for one `(effect_key, handle)` pair on the default
    /// app data lane.
    ///
    /// This keeps terminal mutation state reviewable via `read_layout(cx)` while avoiding repeated
    /// render-triggered follow-up work after the same completion.
    #[cfg(feature = "state-mutation")]
    pub fn take_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
    ) -> bool {
        take_mutation_success_in(self.cx.cx, effect_key, handle)
    }

    /// Default grouped invalidation path for app-facing query state when the caller is already on
    /// `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query<T: Any + Send + Sync + 'static>(self, key: fret_query::QueryKey<T>) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
    }

    /// Default grouped namespace invalidation path for app-facing query state when the caller is
    /// already on `AppUi`.
    #[cfg(feature = "state-query")]
    pub fn invalidate_query_namespace(self, namespace: &'static str) {
        let _ = fret_query::with_query_client(self.cx.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
    }

    /// Invalidate one query key exactly once after a mutation reports success on the default app
    /// data lane.
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
        if !take_mutation_success_in(self.cx.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.cx.app, |client, app| {
            client.invalidate(app, key);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
        true
    }

    /// Invalidate one query namespace exactly once after a mutation reports success on the
    /// default app data lane.
    #[cfg(all(feature = "state-query", feature = "state-mutation"))]
    pub fn invalidate_query_namespace_after_mutation_success<TIn: 'static, TOut: 'static>(
        self,
        effect_key: u64,
        handle: &fret_mutation::MutationHandle<TIn, TOut>,
        namespace: &'static str,
    ) -> bool {
        if !take_mutation_success_in(self.cx.cx, effect_key, handle) {
            return false;
        }

        let _ = fret_query::with_query_client(self.cx.cx.app, |client, _app| {
            client.invalidate_namespace(namespace);
        });
        self.cx.cx.app.request_redraw(self.cx.cx.window);
        true
    }
}

/// Grouped selector/query helpers for extracted app-render helpers on the default app surface.
#[doc(hidden)]
pub struct AppRenderData<'cx, 'a> {
    #[allow(dead_code)]
    cx: &'cx mut ElementContext<'a, crate::app::App>,
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
        take_mutation_completion_in(self.cx, effect_key, handle)
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
