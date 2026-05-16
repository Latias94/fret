//! Runtime table-column visibility helpers for IMUI table authoring.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, UiHost};

use super::TableColumn;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableColumnVisibilityOverride {
    id: Arc<str>,
    visible: bool,
}

/// Model state for runtime table-column visibility.
///
/// This intentionally stays policy-only: it maps stable column ids to visible flags and then
/// produces a new `TableColumn` list. Persistence, header menus, and freeze panes are separate
/// table policies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiTableColumnVisibilityState {
    overrides: Vec<TableColumnVisibilityOverride>,
}

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

    pub fn apply_to_columns(&self, columns: &[TableColumn]) -> Vec<TableColumn> {
        columns
            .iter()
            .cloned()
            .map(|mut column| {
                if let Some(id) = column.id.as_deref() {
                    if let Some(visible) = self.visibility_for(id) {
                        column.visible = visible;
                    }
                }
                column
            })
            .collect()
    }
}

/// Returns a controllable visibility model for an immediate table column set.
pub fn table_column_visibility_use_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled: Option<Model<ImUiTableColumnVisibilityState>>,
    default_value: impl FnOnce() -> ImUiTableColumnVisibilityState,
) -> crate::primitives::controllable_state::ControllableModel<ImUiTableColumnVisibilityState> {
    crate::primitives::controllable_state::use_controllable_model(cx, controlled, default_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Px;

    #[test]
    fn visibility_state_applies_runtime_overrides_by_stable_column_id() {
        let columns = vec![
            TableColumn::fill("Name###name"),
            TableColumn::px("Status###status", Px(96.0)),
            TableColumn::px("Owner###owner", Px(88.0)),
        ];
        let state = ImUiTableColumnVisibilityState::new([
            (Arc::from("status"), false),
            (Arc::from("owner"), true),
        ]);

        let applied = state.apply_to_columns(&columns);

        assert!(applied[0].visible);
        assert!(!applied[1].visible);
        assert!(applied[2].visible);
        assert_eq!(applied[1].id.as_deref(), Some("status"));
        assert_eq!(state.visibility_for("status"), Some(false));
    }

    #[test]
    fn visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility() {
        let columns = vec![
            TableColumn::fill("Name###name"),
            TableColumn::px("Static Hidden###hidden", Px(96.0)).hidden(),
            TableColumn::unlabeled(super::super::TableColumnWidth::px(Px(64.0))),
        ];
        let state = ImUiTableColumnVisibilityState::new([(Arc::from("name"), false)]);

        let applied = state.apply_to_columns(&columns);

        assert!(!applied[0].visible);
        assert!(!applied[1].visible);
        assert!(applied[2].visible);
    }

    #[test]
    fn visibility_state_toggle_uses_current_override_or_default_visibility() {
        let mut state = ImUiTableColumnVisibilityState::default();

        assert!(!state.toggle("status", true));
        assert_eq!(state.visibility_for("status"), Some(false));
        assert!(state.toggle("status", true));
        assert_eq!(state.visibility_for("status"), Some(true));
        assert_eq!(state.remove("status"), Some(true));
        assert!(state.visibility_for("status").is_none());
    }
}
