use std::any::Any;
use std::hash::Hash;
use std::panic::Location;

use slotmap::Key as _;

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
        self.deps.push_token(model.id().data().as_ffi());
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

            let cached = cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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
            cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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

            let cached = cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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
            cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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

            let cached = cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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
            cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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

            let cached = cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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
            cx.root_state(SelectorMemoState::<Deps, TValue>::new, |state| {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        any::{Any, TypeId},
        collections::{HashMap, HashSet},
    };

    use fret_core::{AppWindowId, Point, PointerId, Rect};
    use fret_runtime::{
        ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
        DragSessionId, Effect, EffectSink, FrameId, GlobalsHost, ImageUploadToken, ModelHost,
        ModelId, ModelStore, ModelsHost, ShareSheetToken, TickId, TimeHost, TimerToken,
    };
    use fret_ui::ElementRuntime;

    #[derive(Default)]
    struct TestUiHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        models: ModelStore,
        commands: CommandRegistry,
        redraws: HashSet<AppWindowId>,
        effects: Vec<Effect>,
        drags: HashMap<PointerId, DragSession>,
        next_drag_session_id: u64,
        tick_id: TickId,
        frame_id: FrameId,
        next_timer_token: u64,
        next_clipboard_token: u64,
        next_share_sheet_token: u64,
        next_image_upload_token: u64,
    }

    impl GlobalsHost for TestUiHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals
                .get(&TypeId::of::<T>())
                .and_then(|v| v.downcast_ref::<T>())
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            #[derive(Debug)]
            struct GlobalLeaseMarker;

            struct Guard<T: Any> {
                type_id: TypeId,
                value: Option<T>,
                globals: *mut HashMap<TypeId, Box<dyn Any>>,
            }

            impl<T: Any> Drop for Guard<T> {
                fn drop(&mut self) {
                    let Some(value) = self.value.take() else {
                        return;
                    };
                    unsafe {
                        (*self.globals).insert(self.type_id, Box::new(value));
                    }
                }
            }

            let type_id = TypeId::of::<T>();
            let existing = self
                .globals
                .insert(type_id, Box::new(GlobalLeaseMarker) as Box<dyn Any>);

            let existing = match existing {
                None => None,
                Some(v) => {
                    if v.is::<GlobalLeaseMarker>() {
                        panic!("global already leased: {type_id:?}");
                    }
                    Some(*v.downcast::<T>().expect("global type id must match"))
                }
            };

            let mut guard = Guard::<T> {
                type_id,
                value: Some(existing.unwrap_or_else(init)),
                globals: &mut self.globals as *mut _,
            };

            let result = {
                let value = guard.value.as_mut().expect("guard value exists");
                f(value, self)
            };

            drop(guard);
            result
        }
    }

    impl ModelHost for TestUiHost {
        fn models(&self) -> &ModelStore {
            &self.models
        }

        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }
    }

    impl ModelsHost for TestUiHost {
        fn take_changed_models(&mut self) -> Vec<ModelId> {
            self.models.take_changed_models()
        }
    }

    impl CommandsHost for TestUiHost {
        fn commands(&self) -> &CommandRegistry {
            &self.commands
        }
    }

    impl EffectSink for TestUiHost {
        fn request_redraw(&mut self, window: AppWindowId) {
            self.redraws.insert(window);
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }
    }

    impl TimeHost for TestUiHost {
        fn tick_id(&self) -> TickId {
            self.tick_id
        }

        fn frame_id(&self) -> FrameId {
            self.frame_id
        }

        fn next_timer_token(&mut self) -> TimerToken {
            let token = TimerToken(self.next_timer_token);
            self.next_timer_token = self.next_timer_token.saturating_add(1);
            token
        }

        fn next_clipboard_token(&mut self) -> ClipboardToken {
            let token = ClipboardToken(self.next_clipboard_token);
            self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
            token
        }

        fn next_share_sheet_token(&mut self) -> ShareSheetToken {
            let token = ShareSheetToken(self.next_share_sheet_token);
            self.next_share_sheet_token = self.next_share_sheet_token.saturating_add(1);
            token
        }

        fn next_image_upload_token(&mut self) -> ImageUploadToken {
            let token = ImageUploadToken(self.next_image_upload_token);
            self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
            token
        }
    }

    impl DragHost for TestUiHost {
        fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
            self.drags.get(&pointer_id)
        }

        fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
            self.drags.get_mut(&pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: PointerId) {
            self.drags.remove(&pointer_id);
        }

        fn any_drag_session(&self, predicate: impl FnMut(&DragSession) -> bool) -> bool {
            self.drags.values().any(predicate)
        }

        fn find_drag_pointer_id(
            &self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Option<PointerId> {
            self.drags
                .values()
                .find(|d| predicate(d))
                .map(|d| d.pointer_id)
        }

        fn cancel_drag_sessions(
            &mut self,
            mut predicate: impl FnMut(&DragSession) -> bool,
        ) -> Vec<PointerId> {
            let to_cancel: Vec<PointerId> = self
                .drags
                .values()
                .filter(|d| predicate(d))
                .map(|d| d.pointer_id)
                .collect();
            for pointer_id in &to_cancel {
                self.cancel_drag(*pointer_id);
            }
            to_cancel
        }

        fn begin_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
            let session_id = DragSessionId(self.next_drag_session_id);
            self.drags.insert(
                pointer_id,
                DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
            );
        }

        fn begin_cross_window_drag_with_kind<T: Any>(
            &mut self,
            pointer_id: PointerId,
            kind: DragKindId,
            source_window: AppWindowId,
            start: Point,
            payload: T,
        ) {
            self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
            let session_id = DragSessionId(self.next_drag_session_id);
            self.drags.insert(
                pointer_id,
                DragSession::new_cross_window(
                    session_id,
                    pointer_id,
                    source_window,
                    kind,
                    start,
                    payload,
                ),
            );
        }
    }

    #[test]
    fn deps_builder_model_rev_includes_model_identity_before_revision() {
        let mut host = TestUiHost::default();
        let material = host.models_mut().insert("material".to_string());
        let light = host.models_mut().insert("light".to_string());

        let mut runtime = ElementRuntime::new();
        let mut cx = ElementContext::new_for_root_name(
            &mut host,
            &mut runtime,
            AppWindowId::default(),
            Rect::default(),
            "selector-test",
        );

        let material_sig = {
            let mut deps = DepsBuilder::new(&mut cx);
            deps.model_rev_invalidation(&material, Invalidation::Paint);
            deps.finish()
        };

        let light_sig = {
            let mut deps = DepsBuilder::new(&mut cx);
            deps.model_rev_invalidation(&light, Invalidation::Paint);
            deps.finish()
        };

        assert_eq!(material_sig.tokens().len(), 2);
        assert_eq!(light_sig.tokens().len(), 2);
        assert_eq!(material_sig.tokens()[0], material.id().data().as_ffi());
        assert_eq!(light_sig.tokens()[0], light.id().data().as_ffi());
        assert_eq!(material_sig.tokens()[1], light_sig.tokens()[1]);
        assert_ne!(material_sig, light_sig);
    }
}
