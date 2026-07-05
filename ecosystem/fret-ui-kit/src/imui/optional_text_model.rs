use std::sync::Arc;

use fret_runtime::Model;

/// Narrow interop bridge for immediate-mode optional text selection controls.
///
/// This keeps app-level state handles on the IMUI surface without introducing a broad
/// `IntoModel<T>` conversion story across the whole kit crate.
pub trait IntoImUiOptionalTextModel {
    fn into_imui_optional_text_model(self) -> Model<Option<Arc<str>>>;
}

impl IntoImUiOptionalTextModel for Model<Option<Arc<str>>> {
    fn into_imui_optional_text_model(self) -> Model<Option<Arc<str>>> {
        self
    }
}

impl IntoImUiOptionalTextModel for &Model<Option<Arc<str>>> {
    fn into_imui_optional_text_model(self) -> Model<Option<Arc<str>>> {
        self.clone()
    }
}

impl IntoImUiOptionalTextModel for &mut Model<Option<Arc<str>>> {
    fn into_imui_optional_text_model(self) -> Model<Option<Arc<str>>> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;

    fn accepts_imui_optional_text_model(
        model: impl IntoImUiOptionalTextModel,
    ) -> Model<Option<Arc<str>>> {
        model.into_imui_optional_text_model()
    }

    #[test]
    fn imui_optional_text_model_bridge_accepts_existing_model_reference_shapes() {
        let mut store = ModelStore::default();
        let mut model = store.insert(Some(Arc::<str>::from("mode")));

        let _ = accepts_imui_optional_text_model(model.clone());
        let _ = accepts_imui_optional_text_model(&model);
        let _ = accepts_imui_optional_text_model(&mut model);
    }
}
