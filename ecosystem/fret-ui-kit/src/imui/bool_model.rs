use fret_runtime::Model;

/// Narrow interop bridge for immediate-mode bool controls backed by `Model<bool>`.
///
/// This keeps the IMUI authoring surface compatible with app-level state handles without
/// introducing a broad `IntoModel<T>` conversion story across the whole kit crate.
pub trait IntoImUiBoolModel {
    fn into_imui_bool_model(self) -> Model<bool>;
}

impl IntoImUiBoolModel for Model<bool> {
    fn into_imui_bool_model(self) -> Model<bool> {
        self
    }
}

impl IntoImUiBoolModel for &Model<bool> {
    fn into_imui_bool_model(self) -> Model<bool> {
        self.clone()
    }
}

impl IntoImUiBoolModel for &mut Model<bool> {
    fn into_imui_bool_model(self) -> Model<bool> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;

    fn accepts_imui_bool_model(model: impl IntoImUiBoolModel) -> Model<bool> {
        model.into_imui_bool_model()
    }

    #[test]
    fn imui_bool_model_bridge_accepts_existing_model_reference_shapes() {
        let mut store = ModelStore::default();
        let mut model = store.insert(false);

        let _ = accepts_imui_bool_model(model.clone());
        let _ = accepts_imui_bool_model(&model);
        let _ = accepts_imui_bool_model(&mut model);
    }
}
