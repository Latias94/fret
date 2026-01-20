use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::CommandId;

/// Window-scoped per-command enabled/disabled overrides published by the app layer.
///
/// This is a data-only integration seam used by cross-surface command gating (menus, command
/// palette, shortcuts) without depending on UI-kit or app-specific model types.
///
/// Semantics:
/// - `None`: no override; fall back to `when` evaluation and other gating.
/// - `Some(true)`: force enabled (still may be disabled by other gating).
/// - `Some(false)`: force disabled.
#[derive(Debug, Default)]
pub struct WindowCommandEnabledService {
    by_window: HashMap<AppWindowId, HashMap<CommandId, bool>>,
}

impl WindowCommandEnabledService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&HashMap<CommandId, bool>> {
        self.by_window.get(&window)
    }

    pub fn enabled(&self, window: AppWindowId, command: &CommandId) -> Option<bool> {
        self.by_window
            .get(&window)
            .and_then(|m| m.get(command).copied())
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, enabled: HashMap<CommandId, bool>) {
        self.by_window.insert(window, enabled);
    }

    pub fn set_enabled(&mut self, window: AppWindowId, command: CommandId, enabled: bool) {
        self.by_window
            .entry(window)
            .or_default()
            .insert(command, enabled);
    }

    pub fn clear_command(&mut self, window: AppWindowId, command: &CommandId) {
        let Some(map) = self.by_window.get_mut(&window) else {
            return;
        };
        map.remove(command);
        if map.is_empty() {
            self.by_window.remove(&window);
        }
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_default_is_none() {
        let svc = WindowCommandEnabledService::default();
        assert_eq!(
            svc.enabled(AppWindowId::default(), &CommandId::from("app.preferences")),
            None
        );
    }

    #[test]
    fn set_and_clear_command() {
        let mut svc = WindowCommandEnabledService::default();
        let window = AppWindowId::default();
        let cmd = CommandId::from("app.preferences");

        svc.set_enabled(window, cmd.clone(), false);
        assert_eq!(svc.enabled(window, &cmd), Some(false));

        svc.set_enabled(window, cmd.clone(), true);
        assert_eq!(svc.enabled(window, &cmd), Some(true));

        svc.clear_command(window, &cmd);
        assert_eq!(svc.enabled(window, &cmd), None);
        assert!(svc.snapshot(window).is_none());
    }
}
