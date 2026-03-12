use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_forms(
    cx: &mut UiCx<'_>,
    _text_input: Model<String>,
    _text_area: Model<String>,
    _checkbox: Model<bool>,
    _switch: Model<bool>,
) -> Vec<AnyElement> {
    pages::preview_forms(cx)
}
