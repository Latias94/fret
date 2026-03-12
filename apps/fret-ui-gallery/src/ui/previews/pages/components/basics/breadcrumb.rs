use super::super::super::super::super::*;
use fret::UiCx;

pub(in crate::ui) fn preview_breadcrumb(
    cx: &mut UiCx<'_>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_breadcrumb(cx)
}
