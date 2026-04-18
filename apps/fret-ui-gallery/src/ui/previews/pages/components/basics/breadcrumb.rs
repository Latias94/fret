use super::super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_breadcrumb(
    cx: &mut AppComponentCx<'_>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    pages::preview_breadcrumb(cx)
}
