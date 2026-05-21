use fret_ui::UiHost;

use super::widget_tail::PointerCaptureReleaseCx;

pub(super) trait PendingNodeDragActivationCx<H: UiHost>: PointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
}
