use std::collections::HashMap;
use std::sync::Arc;

use fret_core::AppWindowId;

use crate::CommandId;

/// Window-scoped per-command "action availability" snapshots published by the UI layer.
///
/// This is a data-only integration seam used by cross-surface command gating (menus, command
/// palette, shortcut help) without depending on `fret-ui` internals.
///
/// Semantics:
/// - `None`: no availability information is published for this window/command (treat as unknown).
/// - `Some(true)`: command is available along the current dispatch path.
/// - `Some(false)`: command is unavailable (blocked or not reachable) along the current dispatch path.
#[derive(Debug, Default)]
pub struct WindowCommandActionAvailabilityService {
    by_window: HashMap<AppWindowId, Arc<HashMap<CommandId, bool>>>,
}

impl WindowCommandActionAvailabilityService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&HashMap<CommandId, bool>> {
        self.by_window.get(&window).map(|m| m.as_ref())
    }

    pub fn snapshot_arc(&self, window: AppWindowId) -> Option<Arc<HashMap<CommandId, bool>>> {
        self.by_window.get(&window).cloned()
    }

    pub fn available(&self, window: AppWindowId, command: &CommandId) -> Option<bool> {
        self.by_window
            .get(&window)
            .and_then(|m| m.get(command).copied())
    }

    pub fn set_snapshot(&mut self, window: AppWindowId, availability: HashMap<CommandId, bool>) {
        self.by_window.insert(window, Arc::new(availability));
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_default_is_none() {
        let svc = WindowCommandActionAvailabilityService::default();
        assert_eq!(
            svc.available(AppWindowId::default(), &CommandId::from("app.preferences")),
            None
        );
    }

    #[test]
    fn set_snapshot_publishes_values() {
        let mut svc = WindowCommandActionAvailabilityService::default();
        let window = AppWindowId::default();

        let mut snapshot: HashMap<CommandId, bool> = HashMap::new();
        snapshot.insert(CommandId::from("x"), true);
        snapshot.insert(CommandId::from("y"), false);

        svc.set_snapshot(window, snapshot);

        assert_eq!(svc.available(window, &CommandId::from("x")), Some(true));
        assert_eq!(svc.available(window, &CommandId::from("y")), Some(false));
        assert_eq!(svc.available(window, &CommandId::from("z")), None);
    }
}
