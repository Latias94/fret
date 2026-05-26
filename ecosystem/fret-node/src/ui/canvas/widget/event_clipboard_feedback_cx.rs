use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::low_level_adapter::CanvasPaintInvalidationCx;

pub(super) trait ClipboardFeedbackCx<H: UiHost>: CanvasPaintInvalidationCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
