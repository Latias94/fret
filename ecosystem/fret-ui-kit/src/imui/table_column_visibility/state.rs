use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::imui::TableColumn;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableColumnVisibilityOverride {
    id: Arc<str>,
    visible: bool,
}

/// Model state for runtime table-column visibility.
///
/// This intentionally stays policy-only: it maps stable column ids to visible flags and then
/// produces a new `TableColumn` list. Persistence, freeze panes, and durable column storage are
/// separate table policies.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImUiTableColumnVisibilityState {
    overrides: Vec<TableColumnVisibilityOverride>,
}

/// Persistence-friendly snapshot of runtime table-column visibility overrides.
///
/// This is only a data shape. Callers own where and when it is stored, and the IMUI helper keeps
/// using caller-owned `ImUiTableColumnVisibilityState` models at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TableColumnVisibilitySnapshot {
    #[serde(default)]
    pub columns: Vec<TableColumnVisibilityEntry>,
}

/// One stable column visibility override inside [`TableColumnVisibilitySnapshot`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableColumnVisibilityEntry {
    #[serde(rename = "id")]
    pub column_id: String,
    #[serde(rename = "visible")]
    pub is_visible: bool,
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

    pub fn snapshot(&self) -> TableColumnVisibilitySnapshot {
        TableColumnVisibilitySnapshot {
            columns: self
                .overrides
                .iter()
                .filter(|entry| !entry.id.is_empty())
                .map(|entry| TableColumnVisibilityEntry {
                    column_id: entry.id.to_string(),
                    is_visible: entry.visible,
                })
                .collect(),
        }
    }

    pub fn from_snapshot(snapshot: TableColumnVisibilitySnapshot) -> Self {
        let mut state = Self::default();
        for entry in snapshot.columns {
            state.set_visible(entry.column_id, entry.is_visible);
        }
        state
    }

    pub fn replace_from_snapshot(&mut self, snapshot: TableColumnVisibilitySnapshot) {
        self.clear();
        for entry in snapshot.columns {
            self.set_visible(entry.column_id, entry.is_visible);
        }
    }

    pub fn apply_to_columns(&self, columns: &[TableColumn]) -> Vec<TableColumn> {
        columns
            .iter()
            .cloned()
            .map(|mut column| {
                if let Some(id) = column.id()
                    && let Some(visible) = self.visibility_for(id)
                {
                    column.set_visible_for_policy(visible);
                }
                column
            })
            .collect()
    }
}

impl TableColumnVisibilitySnapshot {
    pub fn new<I, S>(columns: I) -> Self
    where
        I: IntoIterator<Item = (S, bool)>,
        S: Into<String>,
    {
        let mut snapshot = Self::default();
        for (id, visible) in columns {
            let id = id.into();
            if id.is_empty() {
                continue;
            }
            snapshot.columns.push(TableColumnVisibilityEntry {
                column_id: id,
                is_visible: visible,
            });
        }
        snapshot
    }

    pub fn columns(&self) -> &[TableColumnVisibilityEntry] {
        &self.columns
    }

    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

impl TableColumnVisibilityEntry {
    pub fn new(id: impl Into<String>, visible: bool) -> Self {
        Self {
            column_id: id.into(),
            is_visible: visible,
        }
    }

    pub fn id(&self) -> &str {
        self.column_id.as_str()
    }

    pub fn visible(&self) -> bool {
        self.is_visible
    }
}
