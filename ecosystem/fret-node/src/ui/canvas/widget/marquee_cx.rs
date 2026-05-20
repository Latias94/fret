use fret_ui::UiHost;

use super::widget_tail::PointerCaptureReleaseCx;

pub(super) trait MarqueeCx<H: UiHost>: PointerCaptureReleaseCx<H> {
    fn host(&mut self) -> &mut H;
    fn capture_self_pointer(&mut self);
}
