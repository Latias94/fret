use fret_runtime::Model;

/// Narrow interop bridge for immediate-mode f32 value controls backed by `Model<f32>`.
///
/// This keeps app-level state handles on the IMUI surface without introducing a broad
/// `IntoModel<T>` conversion story across the whole kit crate.
pub trait IntoImUiFloatModel {
    fn into_imui_float_model(self) -> Model<f32>;
}

impl IntoImUiFloatModel for Model<f32> {
    fn into_imui_float_model(self) -> Model<f32> {
        self
    }
}

impl IntoImUiFloatModel for &Model<f32> {
    fn into_imui_float_model(self) -> Model<f32> {
        self.clone()
    }
}

impl IntoImUiFloatModel for &mut Model<f32> {
    fn into_imui_float_model(self) -> Model<f32> {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_runtime::ModelStore;

    fn accepts_imui_float_model(model: impl IntoImUiFloatModel) -> Model<f32> {
        model.into_imui_float_model()
    }

    #[test]
    fn imui_float_model_bridge_accepts_existing_model_reference_shapes() {
        let mut store = ModelStore::default();
        let mut model = store.insert(1.0f32);

        let _ = accepts_imui_float_model(model.clone());
        let _ = accepts_imui_float_model(&model);
        let _ = accepts_imui_float_model(&mut model);
    }
}
