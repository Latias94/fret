use std::sync::Arc;

use fret_runtime::Model;

/// Narrow interop bridge for single-selection widgets that store their value in
/// `Model<Option<Arc<str>>>`.
///
/// This stays intentionally focused on the current high-frequency authoring path (`Tabs`,
/// `ToggleGroup`, and other single-select controls) so default app code can pass local-state
/// handles directly without expanding a generic `IntoModel<T>` story across the whole crate.
pub trait IntoOptionalTextValueModel {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>>;
}

impl IntoOptionalTextValueModel for Model<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self
    }
}

impl IntoOptionalTextValueModel for &Model<Option<Arc<str>>> {
    fn into_optional_text_value_model(self) -> Model<Option<Arc<str>>> {
        self.clone()
    }
}
