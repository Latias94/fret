use std::{
    any::Any,
    panic::{AssertUnwindSafe, Location, catch_unwind, resume_unwind},
};

use super::error::ModelUpdateError;
use super::handle::Model;
use super::store::ModelStore;

pub struct ModelCx<'a, H: ModelHost + ?Sized> {
    pub(super) host: &'a mut H,
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
