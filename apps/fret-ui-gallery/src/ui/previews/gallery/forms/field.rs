use super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_field(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    pages::preview_field(cx)
}
