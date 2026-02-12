use super::super::super::super::*;

pub(in crate::ui) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_combobox(cx, value, open, query)
}
