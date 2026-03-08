use super::*;

pub(super) fn invalidate_motion<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
