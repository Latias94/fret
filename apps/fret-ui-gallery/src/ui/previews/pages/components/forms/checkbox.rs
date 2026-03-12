use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_checkbox(cx: &mut UiCx<'_>, _model: Model<bool>) -> Vec<AnyElement> {
    pages::preview_checkbox(cx)
}
