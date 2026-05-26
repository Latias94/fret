use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::low_level_adapter::CanvasPointerCaptureReleaseCx;

pub(super) trait PointerUpReleaseCx<H: UiHost>: CanvasPointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
