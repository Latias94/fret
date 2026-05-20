use fret_ui::UiHost;

pub(super) fn invalidate_motion<H: UiHost>(
    cx: &mut impl super::widget_tail::WidgetPaintInvalidationCx<H>,
) {
    super::widget_tail::invalidate_widget_paint(cx);
}
