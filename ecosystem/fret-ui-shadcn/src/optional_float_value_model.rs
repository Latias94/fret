use fret_runtime::Model;

/// Narrow interop bridge for numeric widgets that store an optional value in a
/// `Model<Option<f32>>`.
pub trait IntoOptionalFloatValueModel {
    fn into_optional_float_value_model(self) -> Model<Option<f32>>;
}

impl IntoOptionalFloatValueModel for Model<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self
    }
}

impl IntoOptionalFloatValueModel for &Model<Option<f32>> {
    fn into_optional_float_value_model(self) -> Model<Option<f32>> {
        self.clone()
    }
}
