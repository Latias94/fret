use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_input(
    cx: &mut UiCx<'_>,
    _value: Model<String>,
    _file_value: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_input(cx)
}
