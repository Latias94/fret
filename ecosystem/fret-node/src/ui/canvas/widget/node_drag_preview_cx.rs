use fret_ui::UiHost;

pub(super) trait NodeDragPreviewCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
}
