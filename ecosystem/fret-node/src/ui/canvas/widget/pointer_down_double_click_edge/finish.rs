use crate::ui::canvas::widget::*;

pub(super) fn finish_double_click<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.stop_propagation();
    paint_invalidation::invalidate_paint(cx);
}
