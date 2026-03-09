use fret_ui::UiHost;

pub(super) fn finish_pointer_up<H: UiHost>(cx: &mut fret_ui::retained_bridge::EventCx<'_, H>) {
    cx.release_pointer_capture();
    super::paint_invalidation::invalidate_paint(cx);
}
