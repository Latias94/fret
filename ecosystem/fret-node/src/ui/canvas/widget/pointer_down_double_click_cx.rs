use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::widget_tail::WidgetHandledCx;

pub(super) trait PointerDownDoubleClickCx<H: UiHost>: WidgetHandledCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
