use std::{any::Any, collections::HashSet, marker::PhantomData};

use slotmap::SlotMap;

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

    pub fn read<H: ModelHost, R>(
        self,
        host: &mut H,
        f: impl FnOnce(&mut H, &T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        host.read(self, f)
    }

    pub fn update<H: ModelHost, R>(
        self,
        host: &mut H,
        f: impl FnOnce(&mut T, &mut ModelCx<'_, H>) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        host.update_model(self, f)
    }

    pub fn revision<H: ModelHost>(self, host: &H) -> Option<u64>
    where
        T: Any,
    {
        host.model_revision(self)
    }

    pub fn get<H: ModelHost>(self, host: &H) -> Option<&T>
    where
        T: Any,
    {
        host.models().get(self)
    }
}

pub struct ModelCx<'a, H: ModelHost + ?Sized> {
    host: &'a mut H,
}

impl<'a, H: ModelHost + ?Sized> ModelCx<'a, H> {
    pub fn app(&mut self) -> &mut H {
        self.host
    }
}

pub trait ModelHost {
    fn models(&self) -> &ModelStore;
    fn models_mut(&mut self) -> &mut ModelStore;

    fn read<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut Self, &T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let lease = self.models_mut().lease(model)?;
        let value = lease.value_ref();
        Ok(f(self, value))
    }

    fn model_revision<T: Any>(&self, model: Model<T>) -> Option<u64> {
        self.models().revision(model)
    }

    fn update_model<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut T, &mut ModelCx<'_, Self>) -> R,
    ) -> Result<R, ModelUpdateError> {
        let mut lease = self.models_mut().lease(model)?;
        let result = {
            let mut cx = ModelCx { host: self };
            f(lease.value_mut(), &mut cx)
        };
        lease.mark_dirty();
        Ok(result)
    }
}

pub struct ModelStore {
    storage: SlotMap<ModelId, ModelEntry>,
    changed: Vec<ModelId>,
    changed_dedup: HashSet<ModelId>,
}

struct ModelEntry {
    value: Option<Box<dyn Any>>,
    revision: u64,
}

impl Default for ModelStore {
    fn default() -> Self {
        Self {
            storage: SlotMap::with_key(),
            changed: Vec::new(),
            changed_dedup: HashSet::new(),
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
            let entry = store.storage.get_mut(self.id).expect("leased id exists");
            assert!(entry.value.is_none(), "model entry should be leased");
            entry.value = Some(value);
            if self.dirty {
                entry.revision = entry.revision.saturating_add(1);
                store.mark_changed(self.id);
            }
        }
    }
}

impl ModelStore {
    fn mark_changed(&mut self, id: ModelId) {
        if self.changed_dedup.insert(id) {
            self.changed.push(id);
        }
    }

    pub fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.changed_dedup.clear();
        std::mem::take(&mut self.changed)
    }

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

    pub fn update<T: Any, R>(
        &mut self,
        model: Model<T>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let mut lease = self.lease(model)?;
        let result = f(lease.value_mut());
        lease.mark_dirty();
        Ok(result)
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
                if let Some(entry) = self.storage.get_mut(model.id)
                    && entry.value.is_none()
                {
                    entry.value = Some(boxed);
                }
                Err(ModelUpdateError::TypeMismatch)
            }
        }
    }
}
