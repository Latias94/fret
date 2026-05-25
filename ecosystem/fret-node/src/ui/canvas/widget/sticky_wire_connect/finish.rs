use fret_ui::UiHost;

pub(super) fn finish_sticky_wire_pointer_down<H: UiHost>(
    cx: &mut impl super::super::low_level_adapter::HandledCanvasPointerCaptureReleaseCx<H>,
) {
    super::super::low_level_adapter::finish_handled_canvas_pointer_capture_release(cx);
}
