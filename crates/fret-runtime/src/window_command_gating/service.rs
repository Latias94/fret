use std::collections::HashMap;

use fret_core::AppWindowId;

use super::snapshot::WindowCommandGatingSnapshot;

/// Token identifying a pushed, overlay-scoped gating override.
///
/// The intent is to allow nested overlays (command palette -> menu -> sub-menu, etc.) to publish
/// independent gating snapshots without clobbering each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WindowCommandGatingToken(u64);

/// Handle identifying a pushed, overlay-scoped gating override.
///
/// Returned by `push_snapshot` so overlays can remove or update only their own snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowCommandGatingHandle {
    window: AppWindowId,
    token: WindowCommandGatingToken,
}

#[derive(Debug, Default)]
struct WindowCommandGatingWindowState {
    base: Option<WindowCommandGatingSnapshot>,
    stack: Vec<(WindowCommandGatingToken, WindowCommandGatingSnapshot)>,
}

#[derive(Debug, Default)]
pub struct WindowCommandGatingService {
    by_window: HashMap<AppWindowId, WindowCommandGatingWindowState>,
    next_token: u64,
}

impl WindowCommandGatingService {
    pub fn snapshot(&self, window: AppWindowId) -> Option<&WindowCommandGatingSnapshot> {
        self.by_window.get(&window).and_then(|state| {
            state
                .stack
                .last()
                .map(|(_, snap)| snap)
                .or(state.base.as_ref())
        })
    }

    pub fn base_snapshot(&self, window: AppWindowId) -> Option<&WindowCommandGatingSnapshot> {
        self.by_window
            .get(&window)
            .and_then(|state| state.base.as_ref())
    }

    /// Sets the base override snapshot for the window.
    ///
    /// Nested overlays should prefer `push_snapshot` so they do not overwrite other overrides.
    pub fn set_base_snapshot(
        &mut self,
        window: AppWindowId,
        snapshot: WindowCommandGatingSnapshot,
    ) {
        let state = self.by_window.entry(window).or_default();
        state.base = Some(snapshot);
        self.gc_window(window);
    }

    /// Sets the base override snapshot for the window.
    ///
    /// Nested overlays should prefer `push_snapshot` so they do not overwrite other overrides.
    pub fn set_snapshot(&mut self, window: AppWindowId, snapshot: WindowCommandGatingSnapshot) {
        self.set_base_snapshot(window, snapshot);
    }

    pub fn clear_base_snapshot(&mut self, window: AppWindowId) {
        if let Some(state) = self.by_window.get_mut(&window) {
            state.base = None;
        }
        self.gc_window(window);
    }

    pub fn clear_snapshot(&mut self, window: AppWindowId) {
        self.clear_base_snapshot(window);
    }

    /// Pushes an overlay-scoped gating snapshot and returns a handle that can later remove it.
    ///
    /// The most recently pushed snapshot wins (`snapshot()` returns the stack top).
    pub fn push_snapshot(
        &mut self,
        window: AppWindowId,
        snapshot: WindowCommandGatingSnapshot,
    ) -> WindowCommandGatingHandle {
        let token = WindowCommandGatingToken(self.next_token.max(1));
        self.next_token = token.0.saturating_add(1);
        let state = self.by_window.entry(window).or_default();
        state.stack.push((token, snapshot));
        WindowCommandGatingHandle { window, token }
    }

    pub fn update_pushed_snapshot(
        &mut self,
        handle: WindowCommandGatingHandle,
        snapshot: WindowCommandGatingSnapshot,
    ) -> bool {
        let Some(state) = self.by_window.get_mut(&handle.window) else {
            return false;
        };
        for (t, s) in &mut state.stack {
            if *t == handle.token {
                *s = snapshot;
                return true;
            }
        }
        false
    }

    pub fn pop_snapshot(
        &mut self,
        handle: WindowCommandGatingHandle,
    ) -> Option<WindowCommandGatingSnapshot> {
        let state = self.by_window.get_mut(&handle.window)?;
        let idx = state.stack.iter().position(|(t, _)| *t == handle.token)?;
        let (_, snapshot) = state.stack.remove(idx);
        self.gc_window(handle.window);
        Some(snapshot)
    }

    pub fn remove_window(&mut self, window: AppWindowId) {
        self.by_window.remove(&window);
    }

    fn gc_window(&mut self, window: AppWindowId) {
        let remove = self
            .by_window
            .get(&window)
            .is_some_and(|state| state.base.is_none() && state.stack.is_empty());
        if remove {
            self.by_window.remove(&window);
        }
    }
}
