use fret_ui::UiHost;

pub(super) trait PendingGroupActivationCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
}
