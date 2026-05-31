use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

#[track_caller]
pub(super) fn draft_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<String> {
    cx.local_model(String::new)
}

#[track_caller]
pub(super) fn error_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<Option<Arc<str>>> {
    cx.local_model(|| None::<Arc<str>>)
}
