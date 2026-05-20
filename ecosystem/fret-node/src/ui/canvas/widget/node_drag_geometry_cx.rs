use fret_ui::UiHost;

pub(super) trait NodeDragGeometryCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
}
