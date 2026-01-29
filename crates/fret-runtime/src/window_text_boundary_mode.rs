use std::collections::HashMap;

use fret_core::AppWindowId;

use crate::TextBoundaryMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WindowTextBoundaryModeToken(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowTextBoundaryModeHandle {
    window: AppWindowId,
    token: WindowTextBoundaryModeToken,
}

#[derive(Debug, Default)]
struct WindowTextBoundaryModeWindowState {
    base: Option<TextBoundaryMode>,
    stack: Vec<(WindowTextBoundaryModeToken, TextBoundaryMode)>,
}

#[derive(Debug, Default)]
pub struct WindowTextBoundaryModeService {
    by_window: HashMap<AppWindowId, WindowTextBoundaryModeWindowState>,
    next_token: u64,
}

impl WindowTextBoundaryModeService {
    pub fn mode(&self, window: AppWindowId) -> Option<TextBoundaryMode> {
        self.by_window
            .get(&window)
            .and_then(|state| state.stack.last().map(|(_, mode)| *mode).or(state.base))
    }

    pub fn set_base_mode(&mut self, window: AppWindowId, mode: TextBoundaryMode) {
        let state = self.by_window.entry(window).or_default();
        state.base = Some(mode);
        self.gc_window(window);
    }

    pub fn clear_base_mode(&mut self, window: AppWindowId) {
        if let Some(state) = self.by_window.get_mut(&window) {
            state.base = None;
        }
        self.gc_window(window);
    }

    pub fn push_mode(
        &mut self,
        window: AppWindowId,
        mode: TextBoundaryMode,
    ) -> WindowTextBoundaryModeHandle {
        let token = WindowTextBoundaryModeToken(self.next_token.max(1));
        self.next_token = token.0.saturating_add(1);
        let state = self.by_window.entry(window).or_default();
        state.stack.push((token, mode));
        WindowTextBoundaryModeHandle { window, token }
    }

    pub fn update_pushed_mode(
        &mut self,
        handle: WindowTextBoundaryModeHandle,
        mode: TextBoundaryMode,
    ) -> bool {
        let Some(state) = self.by_window.get_mut(&handle.window) else {
            return false;
        };
        for (token, entry) in &mut state.stack {
            if *token == handle.token {
                *entry = mode;
                return true;
            }
        }
        false
    }

    pub fn pop_mode(&mut self, handle: WindowTextBoundaryModeHandle) -> Option<TextBoundaryMode> {
        let state = self.by_window.get_mut(&handle.window)?;

        let idx = state
            .stack
            .iter()
            .position(|(token, _)| *token == handle.token)?;
        let (_, removed) = state.stack.remove(idx);
        self.gc_window(handle.window);
        Some(removed)
    }

    fn gc_window(&mut self, window: AppWindowId) {
        if self
            .by_window
            .get(&window)
            .is_some_and(|state| state.base.is_none() && state.stack.is_empty())
        {
            self.by_window.remove(&window);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_wins_over_base() {
        let window = AppWindowId::default();
        let mut svc = WindowTextBoundaryModeService::default();

        svc.set_base_mode(window, TextBoundaryMode::UnicodeWord);
        assert_eq!(svc.mode(window), Some(TextBoundaryMode::UnicodeWord));

        let h1 = svc.push_mode(window, TextBoundaryMode::Identifier);
        assert_eq!(svc.mode(window), Some(TextBoundaryMode::Identifier));

        let h2 = svc.push_mode(window, TextBoundaryMode::UnicodeWord);
        assert_eq!(svc.mode(window), Some(TextBoundaryMode::UnicodeWord));

        svc.pop_mode(h2);
        assert_eq!(svc.mode(window), Some(TextBoundaryMode::Identifier));

        svc.pop_mode(h1);
        assert_eq!(svc.mode(window), Some(TextBoundaryMode::UnicodeWord));
    }

    #[test]
    fn clears_empty_window_state() {
        let window = AppWindowId::default();
        let mut svc = WindowTextBoundaryModeService::default();

        svc.set_base_mode(window, TextBoundaryMode::UnicodeWord);
        svc.clear_base_mode(window);
        assert!(svc.by_window.is_empty());

        let h = svc.push_mode(window, TextBoundaryMode::Identifier);
        svc.pop_mode(h);
        assert!(svc.by_window.is_empty());
    }
}
