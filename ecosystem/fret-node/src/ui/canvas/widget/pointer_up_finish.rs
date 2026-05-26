use fret_ui::UiHost;

pub(super) fn finish_pointer_up<H: UiHost>(
    cx: &mut impl super::low_level_adapter::CanvasPointerCaptureReleaseCx<H>,
) {
    super::low_level_adapter::finish_canvas_pointer_capture_release(cx);
}
