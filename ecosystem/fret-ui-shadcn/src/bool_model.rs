use fret_runtime::Model;

/// Narrow interop bridge for bool-backed widgets that still store their value in a `Model<bool>`.
///
/// This intentionally stays small: it exists so common toggles can accept both explicit
/// `Model<bool>` handles and ecosystem-level local-state handles without introducing a broad
/// `IntoModel<T>` conversion story for every widget.
pub trait IntoBoolModel {
    fn into_bool_model(self) -> Model<bool>;
}

impl IntoBoolModel for Model<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self
    }
}

impl IntoBoolModel for &Model<bool> {
    fn into_bool_model(self) -> Model<bool> {
        self.clone()
    }
}
