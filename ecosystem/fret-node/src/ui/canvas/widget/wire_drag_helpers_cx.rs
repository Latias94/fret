use fret_ui::UiHost;

use super::widget_tail::WidgetPaintInvalidationCx;

pub(super) trait WireDragStartCx<H: UiHost>: WidgetPaintInvalidationCx<H> {
    fn capture_self_pointer(&mut self);
}
