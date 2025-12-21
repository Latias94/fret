use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::Arc,
    time::Duration,
};

use fret_core::{AppWindowId, FrameId, NodeId, Rect, TickId, TimerToken, WindowAnchor};
use slotmap::SlotMap;

use crate::drag::DragKind;
use crate::drag::DragSession;
use crate::{keymap::DefaultKeybinding, when_expr::WhenExpr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandId(pub Arc<str>);

impl CommandId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&'static str> for CommandId {
    fn from(value: &'static str) -> Self {
        Self(Arc::<str>::from(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Redraw(AppWindowId),
    Window(WindowRequest),
    Command {
        window: Option<AppWindowId>,
        command: CommandId,
    },
    /// Framework-level UI invalidation hook (GPUI-style notify).
    ///
    /// This is intentionally UI-runtime-agnostic: the runner/driver decides which UI tree nodes
    /// to invalidate for `Layout` within the given window.
    UiInvalidateLayout {
        window: AppWindowId,
    },
    ClipboardSetText {
        text: String,
    },
    ClipboardGetText {
        window: AppWindowId,
    },
    ViewportInput(fret_core::ViewportInputEvent),
    Dock(fret_core::DockOp),
    ImeAllow {
        window: AppWindowId,
        enabled: bool,
    },
    ImeSetCursorArea {
        window: AppWindowId,
        rect: Rect,
    },
    RequestAnimationFrame(AppWindowId),
    SetTimer {
        window: Option<AppWindowId>,
        token: TimerToken,
        after: Duration,
        repeat: Option<Duration>,
    },
    CancelTimer {
        token: TimerToken,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowRequest {
    Create(CreateWindowRequest),
    Close(AppWindowId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateWindowRequest {
    pub kind: CreateWindowKind,
    pub anchor: Option<WindowAnchor>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateWindowKind {
    DockFloating {
        source_window: AppWindowId,
        panel: fret_core::PanelKey,
    },
    DockRestore {
        logical_window_id: String,
    },
}

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw_requests: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drag: Option<DragSession>,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
            models: ModelStore::default(),
            commands: CommandRegistry::default(),
            redraw_requests: HashSet::new(),
            effects: Vec::new(),
            drag: None,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            next_timer_token: 1,
        }
    }

    pub fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.globals
            .get_mut(&TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    #[track_caller]
    pub fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
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
                // Safety: this guard is only constructed from `App::with_global_mut`, and it
                // outlives the closure execution; `globals` remains valid for the duration.
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
                    panic!(
                        "global already leased: {} ({type_id:?}) at {}",
                        std::any::type_name::<T>(),
                        std::panic::Location::caller()
                    );
                }
                Some(*v.downcast::<T>().expect("global type id must match"))
            }
        };

        let mut guard = Guard::<T> {
            type_id,
            value: Some(existing.unwrap_or_else(init)),
            globals: &mut self.globals as *mut _,
        };

        // Safety: we keep `T` out of `self.globals` until `guard` drops and reinserts it,
        // so it is safe to pass both `&mut T` and `&mut App` to the callback.
        let result = {
            let value = guard.value.as_mut().expect("guard value exists");
            f(value, self)
        };

        drop(guard);
        result
    }

    pub fn models(&self) -> &ModelStore {
        &self.models
    }

    pub fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }

    pub fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    pub fn commands_mut(&mut self) -> &mut CommandRegistry {
        &mut self.commands
    }

    pub fn begin_drag<T: Any>(
        &mut self,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        self.begin_drag_with_kind(DragKind::Custom, source_window, start, payload);
    }

    pub fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new(source_window, kind, start, payload));
    }

    pub fn drag(&self) -> Option<&DragSession> {
        self.drag.as_ref()
    }

    pub fn drag_mut(&mut self) -> Option<&mut DragSession> {
        self.drag.as_mut()
    }

    pub fn end_drag(&mut self) -> Option<DragSession> {
        self.drag.take()
    }

    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

    pub fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw_requests.insert(window);
    }

    /// Runner-owned monotonic tick id (increments once per event-loop turn).
    pub fn tick_id(&self) -> TickId {
        self.tick_id
    }

    /// Runner-owned monotonic frame id (increments on each render/present).
    pub fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    /// Runner-only.
    pub fn set_tick_id(&mut self, tick_id: TickId) {
        self.tick_id = tick_id;
    }

    /// Runner-only.
    pub fn set_frame_id(&mut self, frame_id: FrameId) {
        self.frame_id = frame_id;
    }

    pub fn next_timer_token(&mut self) -> TimerToken {
        let token = TimerToken(self.next_timer_token);
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        token
    }

    pub fn push_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            effect => self.effects.push(effect),
        }
    }

    pub fn flush_effects(&mut self) -> Vec<Effect> {
        let mut effects = std::mem::take(&mut self.effects);
        for window in self.redraw_requests.drain() {
            effects.push(Effect::Redraw(window));
        }
        effects
    }

    pub fn read<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut App, &T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let lease = self.models.lease(model)?;
        let value = lease.value_ref();
        Ok(f(self, value))
    }

    pub fn update<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut App, &mut T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let mut lease = self.models.lease(model)?;
        let result = f(self, lease.value_mut());
        lease.mark_dirty();
        Ok(result)
    }

    pub fn model_revision<T: Any>(&self, model: Model<T>) -> Option<u64> {
        self.models.revision(model)
    }

    pub fn update_model<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut T, &mut ModelCx<'_>) -> R,
    ) -> Result<R, ModelUpdateError> {
        self.update(model, |app, state| {
            let mut cx = ModelCx { app };
            f(state, &mut cx)
        })
    }
}

slotmap::new_key_type! {
    pub struct ModelId;
}

pub struct Model<T> {
    id: ModelId,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Clone for Model<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Model<T> {}

impl<T> Model<T> {
    pub fn id(self) -> ModelId {
        self.id
    }

    pub fn read<R>(
        self,
        app: &mut App,
        f: impl FnOnce(&mut App, &T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        app.read(self, f)
    }

    pub fn update<R>(
        self,
        app: &mut App,
        f: impl FnOnce(&mut T, &mut ModelCx<'_>) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        app.update_model(self, f)
    }

    pub fn revision(self, app: &App) -> Option<u64>
    where
        T: Any,
    {
        app.model_revision(self)
    }

    pub fn get<'a>(self, app: &'a App) -> Option<&'a T>
    where
        T: Any,
    {
        app.models().get(self)
    }
}

pub struct ModelCx<'a> {
    app: &'a mut App,
}

impl<'a> ModelCx<'a> {
    pub fn app(&mut self) -> &mut App {
        self.app
    }
}

pub struct ModelStore {
    storage: SlotMap<ModelId, ModelEntry>,
}

struct ModelEntry {
    value: Option<Box<dyn Any>>,
    revision: u64,
}

impl Default for ModelStore {
    fn default() -> Self {
        Self {
            storage: SlotMap::with_key(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelUpdateError {
    NotFound,
    AlreadyLeased,
    TypeMismatch,
}

struct ModelLease<T: Any> {
    store: *mut ModelStore,
    id: ModelId,
    value: Option<Box<T>>,
    dirty: bool,
}

impl<T: Any> ModelLease<T> {
    fn value_ref(&self) -> &T {
        self.value
            .as_deref()
            .expect("leased model must contain a value")
    }

    fn value_mut(&mut self) -> &mut T {
        self.value
            .as_deref_mut()
            .expect("leased model must contain a value")
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl<T: Any> Drop for ModelLease<T> {
    fn drop(&mut self) {
        let Some(value) = self.value.take() else {
            return;
        };

        unsafe {
            let store = &mut *self.store;
            if let Some(entry) = store.storage.get_mut(self.id) {
                if entry.value.is_none() {
                    entry.value = Some(value);
                    if self.dirty {
                        entry.revision = entry.revision.saturating_add(1);
                    }
                }
            }
        }
    }
}

impl ModelStore {
    pub fn insert<T: Any>(&mut self, value: T) -> Model<T> {
        let id = self.storage.insert(ModelEntry {
            value: Some(Box::new(value)),
            revision: 0,
        });
        Model {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn get<T: Any>(&self, model: Model<T>) -> Option<&T> {
        self.storage
            .get(model.id)?
            .value
            .as_ref()?
            .downcast_ref::<T>()
    }

    pub fn revision<T: Any>(&self, model: Model<T>) -> Option<u64> {
        self.storage.get(model.id).map(|e| e.revision)
    }

    fn lease<T: Any>(&mut self, model: Model<T>) -> Result<ModelLease<T>, ModelUpdateError> {
        let boxed = {
            let entry = self
                .storage
                .get_mut(model.id)
                .ok_or(ModelUpdateError::NotFound)?;
            entry.value.take().ok_or(ModelUpdateError::AlreadyLeased)?
        };

        match boxed.downcast::<T>() {
            Ok(value) => Ok(ModelLease {
                store: self as *mut ModelStore,
                id: model.id,
                value: Some(value),
                dirty: false,
            }),
            Err(boxed) => {
                if let Some(entry) = self.storage.get_mut(model.id) {
                    if entry.value.is_none() {
                        entry.value = Some(boxed);
                    }
                }
                Err(ModelUpdateError::TypeMismatch)
            }
        }
    }
}

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<CommandId, CommandMeta>,
}

#[derive(Debug, Clone)]
pub struct CommandMeta {
    pub title: Arc<str>,
    pub description: Option<Arc<str>>,
    pub category: Option<Arc<str>>,
    pub keywords: Vec<Arc<str>>,
    pub default_keybindings: Vec<DefaultKeybinding>,
    pub when: Option<WhenExpr>,
    pub scope: CommandScope,
    pub hidden: bool,
    pub repeatable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandScope {
    /// Routed through the focused UI node (with bubbling), per ADR 0020.
    Widget,
    /// Handled at the window/app driver boundary (e.g. create/close windows, toggle overlays).
    Window,
    /// Global, cross-window command handled by the app.
    App,
}

impl CommandMeta {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            category: None,
            keywords: Vec::new(),
            default_keybindings: Vec::new(),
            when: None,
            scope: CommandScope::Window,
            hidden: false,
            repeatable: false,
        }
    }

    pub fn with_description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_category(mut self, category: impl Into<Arc<str>>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn with_keywords(
        mut self,
        keywords: impl IntoIterator<Item = impl Into<Arc<str>>>,
    ) -> Self {
        self.keywords = keywords.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_default_keybindings(
        mut self,
        bindings: impl IntoIterator<Item = DefaultKeybinding>,
    ) -> Self {
        self.default_keybindings = bindings.into_iter().collect();
        self
    }

    pub fn with_when(mut self, when: WhenExpr) -> Self {
        self.when = Some(when);
        self
    }

    pub fn with_scope(mut self, scope: CommandScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }

    pub fn repeatable(mut self) -> Self {
        self.repeatable = true;
        self
    }
}

impl CommandRegistry {
    pub fn register(&mut self, id: CommandId, meta: CommandMeta) {
        self.commands.insert(id, meta);
    }

    pub fn get(&self, id: CommandId) -> Option<&CommandMeta> {
        self.commands.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CommandId, &CommandMeta)> {
        self.commands.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus {
    pub node: NodeId,
}
