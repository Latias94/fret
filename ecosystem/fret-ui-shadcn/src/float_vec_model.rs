use fret_runtime::Model;

/// Narrow interop bridge for numeric multi-value widgets that store their values in a
/// `Model<Vec<f32>>`.
pub trait IntoFloatVecModel {
    fn into_float_vec_model(self) -> Model<Vec<f32>>;
}

impl IntoFloatVecModel for Model<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self
    }
}

impl IntoFloatVecModel for &Model<Vec<f32>> {
    fn into_float_vec_model(self) -> Model<Vec<f32>> {
        self.clone()
    }
}
