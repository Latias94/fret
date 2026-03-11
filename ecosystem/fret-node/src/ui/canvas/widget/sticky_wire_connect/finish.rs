use fret_ui::UiHost;

pub(super) fn finish_sticky_wire_pointer_down<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cx.release_pointer_capture();
    cx.stop_propagation();
    super::super::paint_invalidation::invalidate_paint(cx);
}
