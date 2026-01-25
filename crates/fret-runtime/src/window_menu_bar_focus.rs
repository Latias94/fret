use std::collections::HashMap;

use fret_core::AppWindowId;

/// Window-scoped metadata for focusing an in-window menu bar.
///
/// This is a data-only contract between runner shells / UI-kit and the input-dispatch / command
/// gating layer:
/// - When `present == true`, `focus.menu_bar` is expected to be handled for this window.
/// - When `present == false` (or missing), `focus.menu_bar` should be treated as unavailable.
#[derive(Debug, Default)]
pub struct WindowMenuBarFocusService {
    by_window: HashMap<AppWindowId, bool>,
}

impl WindowMenuBarFocusService {
    pub fn present(&self, window: AppWindowId) -> bool {
        self.by_window.get(&window).copied().unwrap_or(false)
    }

    pub fn set_present(&mut self, window: AppWindowId, present: bool) {
        self.by_window.insert(window, present);
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_defaults_to_false() {
        let svc = WindowMenuBarFocusService::default();
        assert!(!svc.present(AppWindowId::default()));
    }

    #[test]
    fn set_present_updates_window() {
        let mut svc = WindowMenuBarFocusService::default();
        let window = AppWindowId::default();
        svc.set_present(window, true);
        assert!(svc.present(window));
        svc.set_present(window, false);
        assert!(!svc.present(window));
    }
}
