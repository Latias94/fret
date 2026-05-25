use fret_ui::UiHost;

use super::low_level_adapter::HandledCanvasPointerCaptureReleaseCx;

pub(super) trait CancelGestureCx<H: UiHost>:
    HandledCanvasPointerCaptureReleaseCx<H>
{
    fn host(&mut self) -> &mut H;
}
