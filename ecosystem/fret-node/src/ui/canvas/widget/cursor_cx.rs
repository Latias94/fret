use fret_core::CursorIcon;
use fret_ui::UiHost;

pub(super) trait CanvasCursorCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn set_cursor_icon(&mut self, icon: CursorIcon);
}
