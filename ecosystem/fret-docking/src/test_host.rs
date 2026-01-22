use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{AppWindowId, Point, PointerId};
use fret_runtime::{ClipboardToken, TimerToken};
use fret_runtime::{
    CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession, DragSessionId, Effect,
    EffectSink, GlobalsHost, ModelHost, ModelId, ModelStore, ModelsHost, TimeHost,
};
use fret_runtime::{FrameId, TickId};

#[derive(Default)]
pub(crate) struct TestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drags: HashMap<PointerId, DragSession>,
    next_drag_session_id: u64,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_image_upload_token: u64,
}

#[allow(dead_code)]
impl TestHost {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_global<T: Any>(&mut self, value: T) {
        GlobalsHost::set_global(self, value);
    }

    pub(crate) fn global<T: Any>(&self) -> Option<&T> {
        GlobalsHost::global(self)
    }

    pub(crate) fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        GlobalsHost::with_global_mut(self, init, f)
    }

    pub(crate) fn models(&self) -> &ModelStore {
        ModelHost::models(self)
    }

    pub(crate) fn models_mut(&mut self) -> &mut ModelStore {
        ModelHost::models_mut(self)
    }

    pub(crate) fn take_changed_models(&mut self) -> Vec<ModelId> {
        ModelsHost::take_changed_models(self)
    }

    pub(crate) fn commands(&self) -> &CommandRegistry {
        CommandsHost::commands(self)
    }

    pub(crate) fn request_redraw(&mut self, window: AppWindowId) {
        EffectSink::request_redraw(self, window);
    }

    pub(crate) fn push_effect(&mut self, effect: Effect) {
        EffectSink::push_effect(self, effect);
    }

    pub(crate) fn tick_id(&self) -> TickId {
        TimeHost::tick_id(self)
    }

    pub(crate) fn frame_id(&self) -> FrameId {
        TimeHost::frame_id(self)
    }

    pub(crate) fn next_timer_token(&mut self) -> TimerToken {
        TimeHost::next_timer_token(self)
    }

    pub(crate) fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        DragHost::drag(self, pointer_id)
    }

    pub(crate) fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        DragHost::drag_mut(self, pointer_id)
    }

    pub(crate) fn cancel_drag(&mut self, pointer_id: PointerId) {
        DragHost::cancel_drag(self, pointer_id)
    }

    pub(crate) fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        DragHost::begin_drag_with_kind(self, pointer_id, kind, source_window, start, payload)
    }

    pub(crate) fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        DragHost::begin_cross_window_drag_with_kind(
            self,
            pointer_id,
            kind,
            source_window,
            start,
            payload,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn advance_frame(&mut self) {
        self.frame_id.0 = self.frame_id.0.saturating_add(1);
    }

    pub(crate) fn take_effects(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.effects)
    }
}

impl GlobalsHost for TestHost {
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

impl ModelsHost for TestHost {
    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestHost {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestHost {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            effect => self.effects.push(effect),
        }
    }
}

impl TimeHost for TestHost {
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

    fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
        let token = fret_runtime::ImageUploadToken(self.next_image_upload_token);
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        token
    }
}

impl DragHost for TestHost {
    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drags.values().any(|d| predicate(d))
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

    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: PointerId) {
        self.drags.remove(&pointer_id);
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
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
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
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

impl ModelHost for TestHost {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}
