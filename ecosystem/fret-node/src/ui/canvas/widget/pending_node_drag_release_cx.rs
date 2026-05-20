use fret_ui::UiHost;

use super::widget_tail::PointerCaptureReleaseCx;

pub(super) trait PendingNodeDragReleaseCx<H: UiHost>: PointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
}
