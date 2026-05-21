use fret_ui::UiHost;

use super::widget_tail::WidgetPaintInvalidationCx;

pub(super) trait NodeResizeMoveCx<H: UiHost>: WidgetPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
}
