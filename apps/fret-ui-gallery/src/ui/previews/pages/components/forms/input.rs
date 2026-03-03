use super::super::super::super::super::*;

pub(in crate::ui) fn preview_input(
    cx: &mut ElementContext<'_, App>,
    _value: Model<String>,
    _file_value: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_input(cx)
}
