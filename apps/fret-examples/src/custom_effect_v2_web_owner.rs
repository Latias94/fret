use fret_runtime::{Model, ModelStore};

pub(crate) trait CustomEffectV2WebControlReset {
    fn reset_controls(&self, owner: &mut CustomEffectV2WebModelOwner<'_>) -> bool;
}

pub(crate) struct CustomEffectV2WebModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> CustomEffectV2WebModelOwner<'a> {
    pub(crate) fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    pub(crate) fn set_model<T: std::any::Any>(&mut self, model: &Model<T>, value: T) -> bool {
        self.models
            .update(model, |current| {
                *current = value;
                true
            })
            .unwrap_or(false)
    }

    pub(crate) fn toggle_surface(&mut self, show: &Model<bool>) -> bool {
        self.models
            .update(show, |v| {
                *v = !*v;
                true
            })
            .unwrap_or(false)
    }

    pub(crate) fn reset_controls<C: CustomEffectV2WebControlReset>(
        &mut self,
        controls: &C,
    ) -> bool {
        controls.reset_controls(self)
    }
}
