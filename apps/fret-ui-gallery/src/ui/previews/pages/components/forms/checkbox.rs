use super::super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_checkbox(cx: &mut AppComponentCx<'_>, _model: Model<bool>) -> Vec<AnyElement> {
    pages::preview_checkbox(cx)
}
