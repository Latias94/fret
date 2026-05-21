use fret_core::{AppWindowId, Rect};
use fret_ui::UiHost;

pub(super) trait AutoPanTimerCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self) -> Rect;
}
