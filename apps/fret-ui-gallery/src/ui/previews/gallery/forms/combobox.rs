use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_combobox(
    cx: &mut UiCx<'_>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_combobox(cx, value, open, query)
}
