use fret_core::Rect;
use fret_ui::UiHost;

use super::widget_tail::WidgetPaintInvalidationCx;

pub(super) trait GroupPreviewMoveCx<H: UiHost>: WidgetPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
    fn bounds(&self) -> Rect;
}
