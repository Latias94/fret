use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use fret_core::{AppWindowId, NodeId};
use slotmap::SlotMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Effect {
    Redraw(AppWindowId),
    Window(WindowRequest),
    Command(CommandId),
    ViewportInput(fret_core::ViewportInputEvent),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowRequest {
    Create(CreateWindowRequest),
    Close(AppWindowId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreateWindowRequest {
    pub kind: CreateWindowKind,
    pub anchor: Option<WindowAnchor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CreateWindowKind {
    DockFloating {
        source_window: AppWindowId,
        panel: fret_core::PanelId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowAnchor {
    pub window: AppWindowId,
    pub position: fret_core::Point,
}

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw_requests: HashSet<AppWindowId>,
    effects: Vec<Effect>,
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

    pub fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw_requests.insert(window);
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

    pub fn update_model<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut T, &mut ModelCx<'_>) -> R,
    ) -> Result<R, ModelUpdateError> {
        let mut lease = self.models.lease(model)?;
        let result = {
            let mut cx = ModelCx { app: self };
            f(lease.value_mut(), &mut cx)
        };
        drop(lease);
        Ok(result)
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
}

impl<T: Any> ModelLease<T> {
    fn value_mut(&mut self) -> &mut T {
        self.value
            .as_deref_mut()
            .expect("leased model must contain a value")
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
                }
            }
        }
    }
}

impl ModelStore {
    pub fn insert<T: Any>(&mut self, value: T) -> Model<T> {
        let id = self.storage.insert(ModelEntry {
            value: Some(Box::new(value)),
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

    pub fn update<T: Any>(&mut self, model: Model<T>, f: impl FnOnce(&mut T)) {
        let Some(entry) = self.storage.get_mut(model.id) else {
            return;
        };
        let Some(any) = entry.value.as_deref_mut() else {
            return;
        };
        let Some(value) = any.downcast_mut::<T>() else {
            return;
        };
        f(value);
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
    pub title: &'static str,
    pub description: Option<&'static str>,
}

impl CommandRegistry {
    pub fn register(&mut self, id: CommandId, meta: CommandMeta) {
        self.commands.insert(id, meta);
    }

    pub fn get(&self, id: CommandId) -> Option<&CommandMeta> {
        self.commands.get(&id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus {
    pub node: NodeId,
}
