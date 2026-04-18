use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_combobox(
    cx: &mut AppComponentCx<'_>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_combobox(cx, value, open, query)
}
