use fret_ui::UiHost;

pub(super) trait PointerDownStateCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
}
