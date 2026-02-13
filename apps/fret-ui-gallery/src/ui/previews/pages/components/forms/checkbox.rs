use super::super::super::super::super::*;

pub(in crate::ui) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_checkbox(cx, model)
}
