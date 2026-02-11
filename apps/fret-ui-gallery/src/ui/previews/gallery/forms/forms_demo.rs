use super::super::super::super::*;

pub(in crate::ui) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_forms(cx, text_input, text_area, checkbox, switch)
}
