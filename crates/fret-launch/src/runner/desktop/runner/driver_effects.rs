use super::WinitRunner;
use super::macos_cursor::dock_tearoff_log;

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_viewport_input_effect(&mut self, event: fret_core::ViewportInputEvent) {
        self.driver.viewport_input(&mut self.app, event);
    }

    pub(super) fn handle_dock_effect(&mut self, op: fret_core::DockOp) {
        if matches!(op, fret_core::DockOp::RequestFloatPanelToNewWindow { .. }) {
            dock_tearoff_log(format_args!("[effect-dock] {:?}", op));
        }
        self.driver.dock_op(&mut self.app, op);
    }
}
