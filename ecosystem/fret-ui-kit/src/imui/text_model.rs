use fret_runtime::Model;

/// Narrow interop bridge for immediate-mode text controls backed by `Model<String>`.
///
/// This keeps app-level state handles on the IMUI surface without introducing a broad
/// `IntoModel<T>` conversion story across the whole kit crate.
pub trait IntoImUiTextModel {
    fn into_imui_text_model(self) -> Model<String>;
}

impl IntoImUiTextModel for Model<String> {
    fn into_imui_text_model(self) -> Model<String> {
        self
    }
}

impl IntoImUiTextModel for &Model<String> {
    fn into_imui_text_model(self) -> Model<String> {
        self.clone()
    }
}

impl IntoImUiTextModel for &mut Model<String> {
    fn into_imui_text_model(self) -> Model<String> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;

    fn accepts_imui_text_model(model: impl IntoImUiTextModel) -> Model<String> {
        model.into_imui_text_model()
    }

    #[test]
    fn imui_text_model_bridge_accepts_existing_model_reference_shapes() {
        let mut store = ModelStore::default();
        let mut model = store.insert(String::from("draft"));

        let _ = accepts_imui_text_model(model.clone());
        let _ = accepts_imui_text_model(&model);
        let _ = accepts_imui_text_model(&mut model);
    }
}
