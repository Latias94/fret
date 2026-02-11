use std::{
    any::Any,
    marker::PhantomData,
    panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind},
    rc::{Rc, Weak},
};

use super::ModelId;
use super::error::ModelUpdateError;
use super::host::{ModelCx, ModelHost};
use super::store::{ModelStore, ModelStoreInner};

/// A reference-counted handle to a typed model stored in a [`ModelStore`].
///
/// This is intentionally gpui-like (`Entity<T>`):
/// - `Model<T>` is a strong handle (cloning increments a per-model strong count).
/// - `WeakModel<T>` can be upgraded back to `Model<T>` if the model is still alive.
/// - When the last strong handle is dropped, the model is removed from the store.
pub struct Model<T> {
    store: ModelStore,
    id: ModelId,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Model<T> {
    pub(super) fn from_store_id(store: ModelStore, id: ModelId) -> Self {
        Self {
            store,
            id,
            _phantom: PhantomData,
        }
    }

    pub fn id(&self) -> ModelId {
        self.id
    }

    pub fn downgrade(&self) -> WeakModel<T> {
        WeakModel {
            store: Rc::downgrade(&self.store.inner),
            id: self.id,
            _phantom: PhantomData,
        }
    }

    pub fn read<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut H, &T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        host.read(self, f)
    }

    #[track_caller]
    pub fn update<H: ModelHost, R>(
        &self,
        host: &mut H,
        f: impl FnOnce(&mut T, &mut ModelCx<'_, H>) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        let changed_at = Location::caller();

        let mut lease = host.models_mut().lease(self)?;
        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| {
                let mut cx = ModelCx { host };
                f(lease.value_mut(), &mut cx)
            }))
        } else {
            Ok({
                let mut cx = ModelCx { host };
                f(lease.value_mut(), &mut cx)
            })
        };

        match result {
            Ok(value) => {
                lease.mark_dirty();
                host.models_mut()
                    .end_lease_with_changed_at(&mut lease, changed_at);
                Ok(value)
            }
            Err(panic) => {
                host.models_mut().end_lease(&mut lease);
                resume_unwind(panic)
            }
        }
    }

    pub fn revision<H: ModelHost>(&self, host: &H) -> Option<u64>
    where
        T: Any,
    {
        host.model_revision(self)
    }

    #[track_caller]
    pub fn notify<H: ModelHost>(&self, host: &mut H) -> Result<(), ModelUpdateError>
    where
        T: Any,
    {
        host.models_mut()
            .notify_with_changed_at(self, Location::caller())
    }

    pub fn read_ref<H: ModelHost, R>(
        &self,
        host: &H,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError>
    where
        T: Any,
    {
        host.models().read(self, f)
    }
}

impl<T> std::fmt::Debug for Model<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model").field("id", &self.id).finish()
    }
}

impl<T> PartialEq for Model<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.store.inner, &other.store.inner) && self.id == other.id
    }
}

impl<T> Eq for Model<T> {}

impl<T> std::hash::Hash for Model<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Rc::as_ptr(&self.store.inner) as usize).hash(state);
        self.id.hash(state);
    }
}

impl<T> Clone for Model<T> {
    fn clone(&self) -> Self {
        self.store.inc_strong(self.id);
        Self {
            store: self.store.clone(),
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for Model<T> {
    fn drop(&mut self) {
        self.store.dec_strong(self.id);
    }
}

pub struct WeakModel<T> {
    store: Weak<ModelStoreInner>,
    id: ModelId,
    _phantom: PhantomData<fn() -> T>,
}

impl<T> std::fmt::Debug for WeakModel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakModel").field("id", &self.id).finish()
    }
}

impl<T> PartialEq for WeakModel<T> {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.store, &other.store) && self.id == other.id
    }
}

impl<T> Eq for WeakModel<T> {}

impl<T> std::hash::Hash for WeakModel<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Weak::as_ptr(&self.store) as usize).hash(state);
        self.id.hash(state);
    }
}

impl<T> Clone for WeakModel<T> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            id: self.id,
            _phantom: PhantomData,
        }
    }
}

impl<T> WeakModel<T> {
    pub fn id(&self) -> ModelId {
        self.id
    }

    pub fn upgrade(&self) -> Option<Model<T>> {
        let store = ModelStore {
            inner: self.store.upgrade()?,
            _not_send: PhantomData,
        };
        store.upgrade_strong(self.id).then(|| Model {
            store,
            id: self.id,
            _phantom: PhantomData,
        })
    }
}
