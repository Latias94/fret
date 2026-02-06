use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};

use fret_core::{AppWindowId, Point};
use fret_runtime::ui_host::{
    CommandsHost, DragHost, EffectSink, GlobalsHost, ModelsHost, TimeHost,
};
use fret_runtime::{
    ClipboardToken, CommandRegistry, DragKindId, DragSession, DragSessionId, Effect, FrameId,
    ModelHost, ModelStore, TickId, TimerToken,
};

#[derive(Default)]
pub(crate) struct TestUiHostImpl {
    pub(crate) globals: HashMap<TypeId, Box<dyn Any>>,
    pub(crate) models: ModelStore,
    pub(crate) commands: CommandRegistry,
    pub(crate) redraw: HashSet<AppWindowId>,
    pub(crate) effects: Vec<Effect>,
    pub(crate) drag: Option<DragSession>,
    pub(crate) tick_id: TickId,
    pub(crate) frame_id: FrameId,
    pub(crate) next_timer_token: u64,
    pub(crate) next_clipboard_token: u64,
    pub(crate) next_image_upload_token: u64,
}

impl GlobalsHost for TestUiHostImpl {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|b| b.downcast_ref::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let type_id = TypeId::of::<T>();
        if !self.globals.contains_key(&type_id) {
            self.globals.insert(type_id, Box::new(init()));
        }

        let boxed = self
            .globals
            .remove(&type_id)
            .expect("global must exist")
            .downcast::<T>()
            .ok()
            .expect("global has wrong type");
        let mut value = *boxed;

        let out = f(&mut value, self);
        self.globals.insert(type_id, Box::new(value));
        out
    }
}

impl ModelHost for TestUiHostImpl {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

impl ModelsHost for TestUiHostImpl {
    fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
        self.models.take_changed_models()
    }
}

impl CommandsHost for TestUiHostImpl {
    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }
}

impl EffectSink for TestUiHostImpl {
    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
}

impl TimeHost for TestUiHostImpl {
    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        TimerToken(self.next_timer_token)
    }

    fn next_clipboard_token(&mut self) -> ClipboardToken {
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        ClipboardToken(self.next_clipboard_token)
    }

    fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        fret_runtime::ImageUploadToken(self.next_image_upload_token)
    }
}

impl DragHost for TestUiHostImpl {
    fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&DragSession> {
        self.drag
            .as_ref()
            .filter(|drag| drag.pointer_id == pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        self.drag.as_ref().is_some_and(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<fret_core::PointerId> {
        self.drag
            .as_ref()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<fret_core::PointerId> {
        let Some(drag) = self.drag.as_ref() else {
            return Vec::new();
        };
        if !predicate(drag) {
            return Vec::new();
        }
        let pointer_id = drag.pointer_id;
        self.drag = None;
        vec![pointer_id]
    }

    fn drag_mut(&mut self, pointer_id: fret_core::PointerId) -> Option<&mut DragSession> {
        self.drag
            .as_mut()
            .filter(|drag| drag.pointer_id == pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
        if self.drag(pointer_id).is_some() {
            self.drag = None;
        }
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new(
            DragSessionId(1),
            pointer_id,
            source_window,
            kind,
            start,
            payload,
        ));
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: fret_core::PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new_cross_window(
            DragSessionId(1),
            pointer_id,
            source_window,
            kind,
            start,
            payload,
        ));
    }
}
