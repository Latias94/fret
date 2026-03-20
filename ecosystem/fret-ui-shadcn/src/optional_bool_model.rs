use fret_runtime::Model;

/// Narrow interop bridge for bool widgets that store an optional value in a `Model<Option<bool>>`.
pub trait IntoOptionalBoolModel {
    fn into_optional_bool_model(self) -> Model<Option<bool>>;
}

impl IntoOptionalBoolModel for Model<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self
    }
}

impl IntoOptionalBoolModel for &Model<Option<bool>> {
    fn into_optional_bool_model(self) -> Model<Option<bool>> {
        self.clone()
    }
}
