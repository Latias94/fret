use fret_runtime::Model;

/// Narrow interop bridge for text-value widgets that still store their value in a `Model<String>`.
///
/// This intentionally stays small: it exists so `Input` / `Textarea` can accept both explicit
/// `Model<String>` handles and ecosystem-level local-state handles without introducing a broad
/// `IntoModel<T>` conversion story for every widget.
pub trait IntoTextValueModel {
    fn into_text_value_model(self) -> Model<String>;
}

impl IntoTextValueModel for Model<String> {
    fn into_text_value_model(self) -> Model<String> {
        self
    }
}

impl IntoTextValueModel for &Model<String> {
    fn into_text_value_model(self) -> Model<String> {
        self.clone()
    }
}
