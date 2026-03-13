use fret_ui::UiHost;
use fret_ui::retained_bridge::{EventCx, Invalidation};

pub(super) fn invalidate_paint<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
