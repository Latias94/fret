use fret_runtime::Model;

/// Narrow interop bridge for widgets that still store their value in a `Model<u8>`.
///
/// This keeps the public authoring surface small while still allowing app-lane local state to
/// participate without widening the crate to a generic `IntoModel<T>` story.
pub trait IntoU8ValueModel {
    fn into_u8_value_model(self) -> Model<u8>;
}

impl IntoU8ValueModel for Model<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self
    }
}

impl IntoU8ValueModel for &Model<u8> {
    fn into_u8_value_model(self) -> Model<u8> {
        self.clone()
    }
}
