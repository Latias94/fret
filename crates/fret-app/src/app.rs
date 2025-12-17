use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
};

use fret_core::NodeId;
use slotmap::SlotMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(pub &'static str);

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
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
}

pub struct ModelStore {
    storage: SlotMap<ModelId, Box<dyn Any>>,
}

impl Default for ModelStore {
    fn default() -> Self {
        Self {
            storage: SlotMap::with_key(),
        }
    }
}

impl ModelStore {
    pub fn insert<T: Any>(&mut self, value: T) -> Model<T> {
        let id = self.storage.insert(Box::new(value));
        Model {
            id,
            _phantom: PhantomData,
        }
    }

    pub fn get<T: Any>(&self, model: Model<T>) -> Option<&T> {
        self.storage.get(model.id)?.downcast_ref::<T>()
    }

    pub fn update<T: Any>(&mut self, model: Model<T>, f: impl FnOnce(&mut T)) {
        let Some(any) = self.storage.get_mut(model.id) else {
            return;
        };
        let Some(value) = any.downcast_mut::<T>() else {
            return;
        };
        f(value);
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
