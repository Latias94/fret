use fret_runtime::{Model, ModelStore};

pub(crate) struct ExternalTextureImportsModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> ExternalTextureImportsModelOwner<'a> {
    pub(crate) fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    pub(crate) fn toggle_surface(&mut self, show: &Model<bool>) -> bool {
        self.models
            .update(show, |show| {
                *show = !*show;
                true
            })
            .unwrap_or(false)
    }
}
