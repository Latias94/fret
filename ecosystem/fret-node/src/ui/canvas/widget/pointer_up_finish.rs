use fret_ui::UiHost;

pub(super) fn finish_pointer_up<H: UiHost>(
    cx: &mut impl super::widget_tail::PointerCaptureReleaseCx<H>,
) {
    super::widget_tail::finish_pointer_capture_release(cx);
}
