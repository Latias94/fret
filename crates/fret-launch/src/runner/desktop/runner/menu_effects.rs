use fret_core::AppWindowId;
use fret_runtime::MenuBar;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_set_menu_bar_effect(
        &mut self,
        window: Option<AppWindowId>,
        menu_bar: MenuBar,
    ) {
        if window.is_none() {
            self.menu_bar = Some(menu_bar.clone());
        }
        #[cfg(windows)]
        {
            let targets: Vec<AppWindowId> = match window {
                Some(window) => vec![window],
                None => self.windows.keys().collect(),
            };
            for window in targets {
                let Some(state) = self.windows.get_mut(window) else {
                    continue;
                };
                let Some(menu) = super::windows_menu::set_window_menu_bar(
                    &self.app,
                    state.window.as_ref(),
                    window,
                    &menu_bar,
                ) else {
                    continue;
                };
                state.os_menu = Some(menu);
            }
        }
        #[cfg(target_os = "macos")]
        {
            let _ = window;
            super::macos_menu::set_app_menu_bar(&self.app, &menu_bar);
        }
        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            let _ = (window, menu_bar);
        }
    }
}
