use std::sync::Arc;

use super::ImUiTableColumnVisibilityState;

impl ImUiTableColumnVisibilityState {
    pub fn new<I, S>(overrides: I) -> Self
    where
        I: IntoIterator<Item = (S, bool)>,
        S: Into<Arc<str>>,
    {
        let mut state = Self::default();
        for (id, visible) in overrides {
            state.set_visible(id, visible);
        }
        state
    }

    pub fn is_empty(&self) -> bool {
        self.overrides.is_empty()
    }

    pub fn len(&self) -> usize {
        self.overrides.len()
    }

    pub fn visibility_for(&self, id: &str) -> Option<bool> {
        self.overrides
            .iter()
            .find(|entry| entry.id.as_ref() == id)
            .map(|entry| entry.visible)
    }

    pub fn is_visible(&self, id: &str, default_visible: bool) -> bool {
        self.visibility_for(id).unwrap_or(default_visible)
    }
}
