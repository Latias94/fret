use super::{WinitAppDriver, WinitRunner, macos_hit_test};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_moved(&mut self) {
        if macos_hit_test::has_active_regions() {
            macos_hit_test::apply_latest_mouse_location();
        }
    }
}
