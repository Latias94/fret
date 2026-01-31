use std::{
    any::Any,
    cell::RefCell,
    collections::HashSet,
    marker::PhantomData,
    panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind},
    rc::{Rc, Weak},
};

use slotmap::SlotMap;

slotmap::new_key_type! {
    pub struct ModelId;
}

#[derive(Debug, Clone, Copy)]
pub struct ModelCreatedDebugInfo {
    pub type_name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelChangedDebugInfo {
    pub type_name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

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
        model: &Model<T>,
        f: impl FnOnce(&mut Self, &T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let mut lease = self.models_mut().lease(model)?;
        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| f(self, lease.value_ref())))
        } else {
            Ok(f(self, lease.value_ref()))
        };

        self.models_mut().end_lease(&mut lease);

        match result {
            Ok(value) => Ok(value),
            Err(panic) => resume_unwind(panic),
        }
    }

    fn model_revision<T: Any>(&self, model: &Model<T>) -> Option<u64> {
        self.models().revision(model)
    }

    #[track_caller]
    fn update_model<T: Any, R>(
        &mut self,
        model: &Model<T>,
        f: impl FnOnce(&mut T, &mut ModelCx<'_, Self>) -> R,
    ) -> Result<R, ModelUpdateError> {
        let changed_at = Location::caller();

        let mut lease = self.models_mut().lease(model)?;
        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| {
                let mut cx = ModelCx { host: self };
                f(lease.value_mut(), &mut cx)
            }))
        } else {
            Ok({
                let mut cx = ModelCx { host: self };
                f(lease.value_mut(), &mut cx)
            })
        };

        match result {
            Ok(value) => {
                lease.mark_dirty();
                self.models_mut()
                    .end_lease_with_changed_at(&mut lease, changed_at);
                Ok(value)
            }
            Err(panic) => {
                self.models_mut().end_lease(&mut lease);
                resume_unwind(panic)
            }
        }
    }
}

pub struct ModelStore {
    inner: Rc<ModelStoreInner>,
    // Models are main-thread only. Enforce this at compile time by making the store (and all
    // derived handles) `!Send` + `!Sync`.
    _not_send: PhantomData<std::rc::Rc<()>>,
}

#[derive(Default)]
struct ModelStoreInner {
    state: RefCell<ModelStoreState>,
}

#[derive(Default)]
struct ModelStoreState {
    storage: SlotMap<ModelId, ModelEntry>,
    changed: Vec<ModelId>,
    changed_dedup: HashSet<ModelId>,
}

struct ModelEntry {
    value: Option<Box<dyn Any>>,
    revision: u64,
    strong: u64,
    pending_drop: bool,
    #[cfg(debug_assertions)]
    created_at: &'static Location<'static>,
    #[cfg(debug_assertions)]
    created_type: &'static str,
    #[cfg(debug_assertions)]
    leased_at: Option<&'static Location<'static>>,
    #[cfg(debug_assertions)]
    leased_type: Option<&'static str>,
    #[cfg(debug_assertions)]
    last_changed_at: Option<&'static Location<'static>>,
    #[cfg(debug_assertions)]
    last_changed_type: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelUpdateError {
    NotFound,
    AlreadyLeased,
    TypeMismatch,
}

pub struct ModelLease<T: Any> {
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
        if self.value.is_some() && !std::thread::panicking() {
            panic!("ModelLease must be ended with ModelStore::end_lease");
        }
    }
}

impl ModelStore {
    #[track_caller]
    fn state(&self) -> std::cell::Ref<'_, ModelStoreState> {
        match self.inner.state.try_borrow() {
            Ok(guard) => guard,
            Err(_) => {
                let caller = Location::caller();
                panic!(
                    "model store is already mutably borrowed (re-entrant borrow) at {}:{}:{}; \
                     this likely indicates a bug where store methods call back into user code while \
                     holding a store borrow",
                    caller.file(),
                    caller.line(),
                    caller.column()
                );
            }
        }
    }

    #[track_caller]
    fn state_mut(&self) -> std::cell::RefMut<'_, ModelStoreState> {
        match self.inner.state.try_borrow_mut() {
            Ok(guard) => guard,
            Err(_) => {
                let caller = Location::caller();
                panic!(
                    "model store is already borrowed (re-entrant borrow_mut) at {}:{}:{}; \
                     this likely indicates a bug where store methods call back into user code while \
                     holding a store borrow",
                    caller.file(),
                    caller.line(),
                    caller.column()
                );
            }
        }
    }

    #[cfg(test)]
    fn state_lock_is_held_for_tests(&self) -> bool {
        self.inner.state.try_borrow_mut().is_err()
    }

    fn mark_changed_locked(state: &mut ModelStoreState, id: ModelId) {
        if state.changed_dedup.insert(id) {
            state.changed.push(id);
        }
    }

    #[cfg(debug_assertions)]
    fn debug_lease_info(&self, id: ModelId) -> Option<(&'static str, &'static Location<'static>)> {
        let state = self.state();
        let entry = state.storage.get(id)?;
        let leased_at = entry.leased_at?;
        let leased_type = entry.leased_type.unwrap_or("<unknown>");
        Some((leased_type, leased_at))
    }

    #[cfg(debug_assertions)]
    fn debug_created_info(
        &self,
        id: ModelId,
    ) -> Option<(&'static str, &'static Location<'static>)> {
        let state = self.state();
        let entry = state.storage.get(id)?;
        Some((entry.created_type, entry.created_at))
    }

    pub fn debug_created_info_for_id(&self, id: ModelId) -> Option<ModelCreatedDebugInfo> {
        #[cfg(debug_assertions)]
        {
            let (type_name, at) = self.debug_created_info(id)?;
            Some(ModelCreatedDebugInfo {
                type_name,
                file: at.file(),
                line: at.line(),
                column: at.column(),
            })
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = id;
            None
        }
    }

    pub fn debug_last_changed_info_for_id(&self, id: ModelId) -> Option<ModelChangedDebugInfo> {
        #[cfg(debug_assertions)]
        {
            let state = self.state();
            let entry = state.storage.get(id)?;
            let at = entry.last_changed_at?;
            let type_name = entry.last_changed_type.unwrap_or("<unknown>");
            Some(ModelChangedDebugInfo {
                type_name,
                file: at.file(),
                line: at.line(),
                column: at.column(),
            })
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = id;
            None
        }
    }

    fn inc_strong(&self, id: ModelId) {
        let mut state = self.state_mut();
        let Some(entry) = state.storage.get_mut(id) else {
            return;
        };
        entry.strong = entry.strong.saturating_add(1);
    }

    fn dec_strong(&self, id: ModelId) {
        // IMPORTANT: do not drop removed values while holding a store borrow.
        //
        // Model values may themselves contain `Model<_>` handles (e.g. composite component state),
        // and dropping those handles re-enters the store to decrement refcounts. Holding a borrow
        // while dropping would trigger a re-entrant borrow panic (and used to deadlock when this
        // was a mutex).
        let removed = {
            let mut state = self.state_mut();
            let should_remove_now = {
                let Some(entry) = state.storage.get_mut(id) else {
                    return;
                };
                entry.strong = entry.strong.saturating_sub(1);
                if entry.strong != 0 {
                    return;
                }
                let should_remove_now = entry.value.is_some();
                if !should_remove_now {
                    entry.pending_drop = true;
                }
                should_remove_now
            };
            Self::mark_changed_locked(&mut state, id);
            should_remove_now.then(|| state.storage.remove(id))
        };

        drop(removed);
    }

    fn upgrade_strong(&self, id: ModelId) -> bool {
        let mut state = self.state_mut();
        let Some(entry) = state.storage.get_mut(id) else {
            return false;
        };
        if entry.strong == 0 {
            return false;
        }
        entry.strong = entry.strong.saturating_add(1);
        true
    }

    pub fn take_changed_models(&mut self) -> Vec<ModelId> {
        let mut state = self.state_mut();
        state.changed_dedup.clear();
        let changed = std::mem::take(&mut state.changed);
        changed
            .into_iter()
            .filter(|&id| {
                state
                    .storage
                    .get(id)
                    .is_some_and(|entry| entry.strong > 0 && entry.value.is_some())
            })
            .collect()
    }

    #[track_caller]
    pub fn insert<T: Any>(&mut self, value: T) -> Model<T> {
        #[cfg(debug_assertions)]
        let caller = Location::caller();
        let mut state = self.state_mut();
        let id = state.storage.insert(ModelEntry {
            value: Some(Box::new(value)),
            revision: 0,
            strong: 1,
            pending_drop: false,
            #[cfg(debug_assertions)]
            created_at: caller,
            #[cfg(debug_assertions)]
            created_type: std::any::type_name::<T>(),
            #[cfg(debug_assertions)]
            leased_at: None,
            #[cfg(debug_assertions)]
            leased_type: None,
            #[cfg(debug_assertions)]
            last_changed_at: None,
            #[cfg(debug_assertions)]
            last_changed_type: None,
        });
        Model {
            store: self.clone(),
            id,
            _phantom: PhantomData,
        }
    }

    pub fn read<T: Any, R>(
        &self,
        model: &Model<T>,
        f: impl FnOnce(&T) -> R,
    ) -> Result<R, ModelUpdateError> {
        // IMPORTANT: do not run user code while holding a store borrow.
        //
        // Model values can contain other `Model<_>` handles. Cloning/dropping those handles
        // re-enters this store, so holding a borrow would trigger a re-entrant borrow panic (and
        // used to deadlock when this was a mutex).
        let mut lease = self.lease_shared(model)?;
        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| f(lease.value_ref())))
        } else {
            Ok(f(lease.value_ref()))
        };

        self.end_lease_shared(&mut lease, None);

        match result {
            Ok(value) => Ok(value),
            Err(panic) => resume_unwind(panic),
        }
    }

    pub fn get_copied<T: Any + Copy>(&self, model: &Model<T>) -> Option<T> {
        match self.read(model, |v| *v) {
            Ok(v) => Some(v),
            Err(ModelUpdateError::NotFound) => None,
            Err(ModelUpdateError::AlreadyLeased) => {
                #[cfg(debug_assertions)]
                if let Some((ty, at)) = self.debug_lease_info(model.id) {
                    panic!(
                        "model is currently leased: id={:?} type={} leased_at={}:{}:{}",
                        model.id,
                        ty,
                        at.file(),
                        at.line(),
                        at.column()
                    );
                }
                panic!("model is currently leased: id={:?}", model.id);
            }
            Err(ModelUpdateError::TypeMismatch) => {
                #[cfg(debug_assertions)]
                if let Some((stored, at)) = self.debug_created_info(model.id) {
                    panic!(
                        "model type mismatch: id={:?} stored_type={} stored_at={}:{}:{} expected_type={}",
                        model.id,
                        stored,
                        at.file(),
                        at.line(),
                        at.column(),
                        std::any::type_name::<T>()
                    );
                }
                panic!("model type mismatch: id={:?}", model.id);
            }
        }
    }

    pub fn get_cloned<T: Any + Clone>(&self, model: &Model<T>) -> Option<T> {
        match self.read(model, |v| v.clone()) {
            Ok(v) => Some(v),
            Err(ModelUpdateError::NotFound) => None,
            Err(ModelUpdateError::AlreadyLeased) => {
                #[cfg(debug_assertions)]
                if let Some((ty, at)) = self.debug_lease_info(model.id) {
                    panic!(
                        "model is currently leased: id={:?} type={} leased_at={}:{}:{}",
                        model.id,
                        ty,
                        at.file(),
                        at.line(),
                        at.column()
                    );
                }
                panic!("model is currently leased: id={:?}", model.id);
            }
            Err(ModelUpdateError::TypeMismatch) => {
                #[cfg(debug_assertions)]
                if let Some((stored, at)) = self.debug_created_info(model.id) {
                    panic!(
                        "model type mismatch: id={:?} stored_type={} stored_at={}:{}:{} expected_type={}",
                        model.id,
                        stored,
                        at.file(),
                        at.line(),
                        at.column(),
                        std::any::type_name::<T>()
                    );
                }
                panic!("model type mismatch: id={:?}", model.id);
            }
        }
    }

    pub fn revision<T: Any>(&self, model: &Model<T>) -> Option<u64> {
        let state = self.state();
        state.storage.get(model.id).map(|e| e.revision)
    }

    #[track_caller]
    pub fn update<T: Any, R>(
        &mut self,
        model: &Model<T>,
        f: impl FnOnce(&mut T) -> R,
    ) -> Result<R, ModelUpdateError> {
        let changed_at = Location::caller();

        let mut lease = self.lease(model)?;
        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| f(lease.value_mut())))
        } else {
            Ok(f(lease.value_mut()))
        };

        match result {
            Ok(value) => {
                lease.mark_dirty();
                self.end_lease_with_changed_at(&mut lease, changed_at);
                Ok(value)
            }
            Err(panic) => {
                self.end_lease(&mut lease);
                resume_unwind(panic)
            }
        }
    }

    #[track_caller]
    fn lease_shared<T: Any>(&self, model: &Model<T>) -> Result<ModelLease<T>, ModelUpdateError> {
        #[cfg(debug_assertions)]
        let caller = Location::caller();
        let boxed = {
            let mut state = self.state_mut();
            let entry = state
                .storage
                .get_mut(model.id)
                .ok_or(ModelUpdateError::NotFound)?;
            if entry.strong == 0 {
                return Err(ModelUpdateError::NotFound);
            }

            match entry.value.take() {
                Some(value) => {
                    #[cfg(debug_assertions)]
                    {
                        entry.leased_at = Some(caller);
                        entry.leased_type = Some(std::any::type_name::<T>());
                    }
                    value
                }
                None => {
                    #[cfg(debug_assertions)]
                    {
                        let leased_type = entry.leased_type.unwrap_or("<unknown>");
                        if let Some(leased_at) = entry.leased_at {
                            eprintln!(
                                "model already leased: id={:?} type={} leased_at={}:{}:{} attempted_at={}:{}:{}",
                                model.id,
                                leased_type,
                                leased_at.file(),
                                leased_at.line(),
                                leased_at.column(),
                                caller.file(),
                                caller.line(),
                                caller.column()
                            );
                        } else {
                            eprintln!(
                                "model already leased: id={:?} type={} attempted_at={}:{}:{} (lease origin unknown)",
                                model.id,
                                leased_type,
                                caller.file(),
                                caller.line(),
                                caller.column()
                            );
                        }
                    }
                    return Err(ModelUpdateError::AlreadyLeased);
                }
            }
        };

        match boxed.downcast::<T>() {
            Ok(value) => Ok(ModelLease {
                id: model.id,
                value: Some(value),
                dirty: false,
            }),
            Err(boxed) => {
                #[cfg(debug_assertions)]
                {
                    let state = self.state();
                    if let Some(entry) = state.storage.get(model.id) {
                        eprintln!(
                            "model type mismatch: id={:?} stored_type={} stored_at={}:{}:{} expected_type={} attempted_at={}:{}:{}",
                            model.id,
                            entry.created_type,
                            entry.created_at.file(),
                            entry.created_at.line(),
                            entry.created_at.column(),
                            std::any::type_name::<T>(),
                            caller.file(),
                            caller.line(),
                            caller.column()
                        );
                    }
                }

                let mut state = self.state_mut();
                if let Some(entry) = state.storage.get_mut(model.id)
                    && entry.value.is_none()
                {
                    entry.value = Some(boxed);
                    #[cfg(debug_assertions)]
                    {
                        entry.leased_at = None;
                        entry.leased_type = None;
                    }
                }
                Err(ModelUpdateError::TypeMismatch)
            }
        }
    }

    pub fn lease<T: Any>(&mut self, model: &Model<T>) -> Result<ModelLease<T>, ModelUpdateError> {
        self.lease_shared(model)
    }

    fn end_lease_shared<T: Any>(
        &self,
        lease: &mut ModelLease<T>,
        changed_at: Option<&'static Location<'static>>,
    ) {
        let Some(value) = lease.value.take() else {
            return;
        };

        // Same borrow-drop rule as `dec_strong`: do not drop removed values while holding a borrow.
        let removed = {
            let mut state = self.state_mut();
            let (mark_dirty, should_remove) = {
                let entry = state.storage.get_mut(lease.id).expect("leased id exists");
                assert!(entry.value.is_none(), "model entry should be leased");
                entry.value = Some(value);
                #[cfg(debug_assertions)]
                {
                    entry.leased_at = None;
                    entry.leased_type = None;
                }
                if lease.dirty {
                    entry.revision = entry.revision.saturating_add(1);
                    #[cfg(debug_assertions)]
                    {
                        entry.last_changed_at = changed_at;
                        entry.last_changed_type = Some(std::any::type_name::<T>());
                    }
                    // NOTE: `entry` holds a mutable borrow of `state`, so defer the `mark_changed` call.
                }
                let should_remove = entry.pending_drop && entry.strong == 0;
                (lease.dirty, should_remove)
            };
            if mark_dirty {
                Self::mark_changed_locked(&mut state, lease.id);
            }
            if should_remove {
                Self::mark_changed_locked(&mut state, lease.id);
                Some(state.storage.remove(lease.id))
            } else {
                None
            }
        };

        drop(removed);
    }

    pub fn end_lease<T: Any>(&mut self, lease: &mut ModelLease<T>) {
        self.end_lease_shared(lease, None);
    }

    pub fn end_lease_with_changed_at<T: Any>(
        &mut self,
        lease: &mut ModelLease<T>,
        changed_at: &'static Location<'static>,
    ) {
        self.end_lease_shared(lease, Some(changed_at));
    }

    pub fn notify_with_changed_at<T: Any>(
        &mut self,
        model: &Model<T>,
        changed_at: &'static Location<'static>,
    ) -> Result<(), ModelUpdateError> {
        let id = model.id;

        let mut state = self.state_mut();
        {
            let Some(entry) = state.storage.get_mut(id) else {
                return Err(ModelUpdateError::NotFound);
            };
            if entry.strong == 0 {
                return Err(ModelUpdateError::NotFound);
            }
            let Some(value) = entry.value.as_ref() else {
                return Err(ModelUpdateError::AlreadyLeased);
            };
            if !value.is::<T>() {
                return Err(ModelUpdateError::TypeMismatch);
            }

            entry.revision = entry.revision.saturating_add(1);
            #[cfg(debug_assertions)]
            {
                entry.last_changed_at = Some(changed_at);
                entry.last_changed_type = Some(std::any::type_name::<T>());
            }
        }

        Self::mark_changed_locked(&mut state, id);
        Ok(())
    }

    #[track_caller]
    pub fn notify<T: Any>(&mut self, model: &Model<T>) -> Result<(), ModelUpdateError> {
        self.notify_with_changed_at(model, Location::caller())
    }
}

impl Clone for ModelStore {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _not_send: PhantomData,
        }
    }
}

impl Default for ModelStore {
    fn default() -> Self {
        Self {
            inner: Rc::new(ModelStoreInner::default()),
            _not_send: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(debug_assertions)]
    #[test]
    fn lease_markers_are_set_and_cleared() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);
        let id = model.id();

        let mut lease = store.lease(&model).unwrap();
        assert!(store.debug_lease_info(id).is_some());

        store.end_lease(&mut lease);
        assert!(store.debug_lease_info(id).is_none());
    }

    #[test]
    fn read_does_not_hold_store_lock_while_running_user_code() {
        #[derive(Clone)]
        struct Outer {
            inner: Model<u32>,
        }

        let mut store = ModelStore::default();
        let inner = store.insert(123_u32);
        let outer = store.insert(Outer {
            inner: inner.clone(),
        });

        let out = store
            .read(&outer, |outer| {
                assert!(
                    !store.state_lock_is_held_for_tests(),
                    "ModelStore::read must not hold a store borrow while running user closures"
                );

                // If `read` regresses to holding the lock while executing the closure, this clone
                // would re-enter the store and could trigger a re-entrant borrow panic (and used to
                // deadlock when this was a mutex). The borrow probe above turns that into a
                // deterministic assertion failure.
                let _inner_clone = outer.inner.clone();

                1u32
            })
            .expect("outer model should be readable");

        assert_eq!(out, 1);
    }

    #[test]
    fn dropping_last_strong_does_not_drop_value_while_holding_store_lock() {
        struct DropProbe {
            store: ModelStore,
        }

        impl Drop for DropProbe {
            fn drop(&mut self) {
                assert!(
                    !self.store.state_lock_is_held_for_tests(),
                    "model value must not be dropped while holding a store borrow"
                );
            }
        }

        let mut store = ModelStore::default();
        let model = store.insert(DropProbe {
            store: store.clone(),
        });

        drop(model);
    }

    #[test]
    fn strong_handle_drop_removes_entry() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);
        let id = model.id();

        assert!(store.state().storage.contains_key(id));
        drop(model);
        assert!(!store.state().storage.contains_key(id));
    }

    #[test]
    fn clone_increments_and_decrements_strong_count() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);
        let id = model.id();

        let model2 = model.clone();
        {
            let state = store.state();
            let entry = state.storage.get(id).unwrap();
            assert_eq!(entry.strong, 2);
        }

        drop(model);
        {
            let state = store.state();
            let entry = state.storage.get(id).unwrap();
            assert_eq!(entry.strong, 1);
        }

        drop(model2);
        assert!(!store.state().storage.contains_key(id));
    }

    #[test]
    fn dropping_last_strong_while_leased_defers_removal_until_end_lease() {
        let mut store = ModelStore::default();
        let model = store.insert(String::from("hello"));
        let id = model.id();

        let mut lease = store.lease(&model).unwrap();
        drop(model);

        {
            let state = store.state();
            let entry = state.storage.get(id).unwrap();
            assert_eq!(entry.strong, 0);
            assert!(entry.value.is_none());
            assert!(entry.pending_drop);
        }

        store.end_lease(&mut lease);
        assert!(!store.state().storage.contains_key(id));
    }

    #[test]
    fn take_changed_models_filters_dropped_entries() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);

        let _ = store.update(&model, |v| *v = 456_u32);
        drop(model);

        let changed = store.take_changed_models();
        assert_eq!(changed.len(), 0);
    }

    #[allow(clippy::mutable_key_type)]
    #[test]
    fn model_equality_and_hash_are_scoped_to_the_store() {
        let mut store_a = ModelStore::default();
        let mut store_b = ModelStore::default();

        let a = store_a.insert(1u32);
        let b = store_b.insert(1u32);

        assert_ne!(a, b);

        let mut set = std::collections::HashSet::new();
        set.insert(a.clone());
        set.insert(b.clone());
        assert_eq!(set.len(), 2);

        let weak_a = a.downgrade();
        let weak_b = b.downgrade();
        assert_ne!(weak_a, weak_b);
    }

    #[test]
    fn notify_marks_changed_and_bumps_revision() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);

        assert_eq!(store.revision(&model), Some(0));
        store.notify(&model).expect("notify should succeed");
        assert_eq!(store.revision(&model), Some(1));

        let changed = store.take_changed_models();
        assert_eq!(changed, vec![model.id()]);
    }

    #[test]
    fn notify_errors_while_leased() {
        let mut store = ModelStore::default();
        let model = store.insert(123_u32);

        let mut lease = store.lease(&model).expect("lease should succeed");
        let err = store.notify(&model).expect_err("notify should fail");
        assert!(matches!(err, ModelUpdateError::AlreadyLeased));

        store.end_lease(&mut lease);
    }
}
