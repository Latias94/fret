use super::super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_input(
    cx: &mut AppComponentCx<'_>,
    _value: Model<String>,
    _file_value: Model<String>,
) -> Vec<AnyElement> {
    pages::preview_input(cx)
}
