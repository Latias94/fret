use super::WinitRunner;

impl<D: super::WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_viewport_input_effect(&mut self, event: fret_core::ViewportInputEvent) {
        self.driver.viewport_input(&mut self.app, event);
    }

    pub(super) fn handle_dock_effect(&mut self, op: fret_core::DockOp) {
        self.driver.dock_op(&mut self.app, op);
    }
}
