use fret_ui::UiHost;

use super::widget_tail::HandledPointerCaptureReleaseCx;

pub(super) trait CancelGestureCx<H: UiHost>: HandledPointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
}
