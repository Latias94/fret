use fret_core::AppWindowId;
use fret_ui::UiHost;

use super::widget_tail::PointerCaptureReleaseCx;

pub(super) trait PointerUpCommitCx<H: UiHost>: PointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}
