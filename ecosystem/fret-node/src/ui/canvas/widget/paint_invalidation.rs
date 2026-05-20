use fret_ui::UiHost;

use super::widget_tail::{WidgetPaintInvalidationCx, invalidate_widget_paint};

pub(super) fn invalidate_paint<H: UiHost>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    invalidate_widget_paint(cx);
}
