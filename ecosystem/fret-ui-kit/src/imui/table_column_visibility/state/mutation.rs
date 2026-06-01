use std::sync::Arc;

use super::{ImUiTableColumnVisibilityState, TableColumnVisibilityOverride};

impl ImUiTableColumnVisibilityState {
    pub fn set_visible(&mut self, id: impl Into<Arc<str>>, visible: bool) {
        let id = id.into();
        if id.is_empty() {
            return;
        }

        if let Some(entry) = self
            .overrides
            .iter_mut()
            .find(|entry| entry.id.as_ref() == id.as_ref())
        {
            entry.visible = visible;
            return;
        }

        self.overrides
            .push(TableColumnVisibilityOverride { id, visible });
    }

    pub fn show(&mut self, id: impl Into<Arc<str>>) {
        self.set_visible(id, true);
    }

    pub fn hide(&mut self, id: impl Into<Arc<str>>) {
        self.set_visible(id, false);
    }

    pub fn toggle(&mut self, id: impl Into<Arc<str>>, default_visible: bool) -> bool {
        let id = id.into();
        if id.is_empty() {
            return default_visible;
        }

        let visible = !self.is_visible(id.as_ref(), default_visible);
        self.set_visible(id, visible);
        visible
    }

    pub fn remove(&mut self, id: &str) -> Option<bool> {
        let index = self
            .overrides
            .iter()
            .position(|entry| entry.id.as_ref() == id)?;
        Some(self.overrides.remove(index).visible)
    }

    pub fn clear(&mut self) {
        self.overrides.clear();
    }
}
