use fret_runtime::Model;

/// Narrow interop bridge for numeric widgets that store a single value in a `Model<f32>`.
pub trait IntoFloatValueModel {
    fn into_float_value_model(self) -> Model<f32>;
}

impl IntoFloatValueModel for Model<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self
    }
}

impl IntoFloatValueModel for &Model<f32> {
    fn into_float_value_model(self) -> Model<f32> {
        self.clone()
    }
}
