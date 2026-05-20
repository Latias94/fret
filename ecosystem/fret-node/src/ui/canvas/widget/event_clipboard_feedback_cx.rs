use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::widget_tail::WidgetPaintInvalidationCx;

pub(super) trait ClipboardFeedbackCx<H: UiHost>: WidgetPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
