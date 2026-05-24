use fret_core::{AppWindowId, Rect};
use fret_ui::UiHost;

pub(in crate::ui::canvas::widget) trait EdgeInsertCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self) -> Rect;
}
