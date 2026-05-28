use super::{
    ImUiTableColumnVisibilityState, TableColumnVisibilityEntry, TableColumnVisibilitySnapshot,
};

impl ImUiTableColumnVisibilityState {
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
}
