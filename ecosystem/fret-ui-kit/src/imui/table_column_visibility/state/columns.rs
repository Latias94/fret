use crate::imui::TableColumn;

use super::ImUiTableColumnVisibilityState;

impl ImUiTableColumnVisibilityState {
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
