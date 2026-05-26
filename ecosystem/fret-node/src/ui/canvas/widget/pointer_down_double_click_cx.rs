use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::low_level_adapter::CanvasHandledCx;

pub(super) trait PointerDownDoubleClickCx<H: UiHost>: CanvasHandledCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
