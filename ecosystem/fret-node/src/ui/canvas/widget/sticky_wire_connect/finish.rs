use fret_ui::UiHost;

pub(super) fn finish_sticky_wire_pointer_down<H: UiHost>(
    cx: &mut impl super::super::widget_tail::HandledPointerCaptureReleaseCx<H>,
) {
    super::super::widget_tail::finish_handled_pointer_capture_release(cx);
}
