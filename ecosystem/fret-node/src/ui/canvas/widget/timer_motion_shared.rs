use fret_ui::UiHost;

pub(super) fn invalidate_motion<H: UiHost>(
    cx: &mut impl super::low_level_adapter::CanvasPaintInvalidationCx<H>,
) {
    super::low_level_adapter::invalidate_canvas_paint(cx);
}
