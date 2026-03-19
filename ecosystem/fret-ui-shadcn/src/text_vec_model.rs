use std::sync::Arc;

use fret_runtime::Model;

/// Narrow interop bridge for multi-select text-value widgets that store their value in a
/// `Model<Vec<Arc<str>>>`.
pub trait IntoTextVecModel {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>>;
}

impl IntoTextVecModel for Model<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self
    }
}

impl IntoTextVecModel for &Model<Vec<Arc<str>>> {
    fn into_text_vec_model(self) -> Model<Vec<Arc<str>>> {
        self.clone()
    }
}
