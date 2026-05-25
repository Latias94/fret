use fret_ui::UiHost;

use super::low_level_adapter::CanvasPointerCaptureReleaseCx;

pub(super) trait PendingNodeDragActivationCx<H: UiHost>:
    CanvasPointerCaptureReleaseCx<H>
{
    fn host(&mut self) -> &mut H;
}
