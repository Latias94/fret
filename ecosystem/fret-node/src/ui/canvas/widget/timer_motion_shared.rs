use super::*;

pub(super) fn invalidate_motion<H: UiHost>(cx: &mut EventCx<'_, H>) {
    super::paint_invalidation::invalidate_paint(cx);
}
