use std::collections::HashMap;

use super::column_sizing_info::ColumnSizingInfoState;
use super::{ColumnDef, ColumnId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnSizingRegion {
    All,
    Left,
    Center,
    Right,
}

/// TanStack-compatible column sizing map: `column_id -> size`.
pub type ColumnSizingState = HashMap<ColumnId, f32>;

pub fn column_size(state: &ColumnSizingState, column: &ColumnId) -> Option<f32> {
    state.get(column).copied()
}

pub fn resolved_column_size<TData>(state: &ColumnSizingState, column: &ColumnDef<TData>) -> f32 {
    let raw = state.get(&column.id).copied().unwrap_or(column.size);
    raw.clamp(column.min_size, column.max_size)
}

pub fn column_can_resize<TData>(options: super::TableOptions, column: &ColumnDef<TData>) -> bool {
    options.enable_column_resizing && column.enable_resizing
}

pub fn start_column_resize(
    info: &mut ColumnSizingInfoState,
    column: &ColumnId,
    pointer_x: f32,
    start_size: f32,
) {
    info.is_resizing_column = Some(column.clone());
    info.start_pointer_x = pointer_x;
    info.start_size = start_size;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::table::TableOptions;

    #[test]
    fn column_size_reads_from_map() {
        let mut state = ColumnSizingState::default();
        state.insert(ColumnId::from("a"), 123.0);

        assert_eq!(column_size(&state, &ColumnId::from("a")), Some(123.0));
        assert_eq!(column_size(&state, &ColumnId::from("b")), None);
    }

    #[test]
    fn resolved_column_size_falls_back_to_column_default_and_clamps() {
        #[derive(Debug)]
        struct Item;

        let col = ColumnDef::<Item>::new("a")
            .size(100.0)
            .min_size(60.0)
            .max_size(80.0);

        let state = ColumnSizingState::default();
        assert_eq!(resolved_column_size(&state, &col), 80.0);

        let mut state = ColumnSizingState::default();
        state.insert(ColumnId::from("a"), 10.0);
        assert_eq!(resolved_column_size(&state, &col), 60.0);
    }

    #[test]
    fn column_can_resize_respects_table_and_column_flags() {
        #[derive(Debug)]
        struct Item;

        let col = ColumnDef::<Item>::new("a").enable_resizing(false);
        assert!(!column_can_resize(TableOptions::default(), &col));

        let mut options = TableOptions::default();
        options.enable_column_resizing = false;
        let col = ColumnDef::<Item>::new("a").enable_resizing(true);
        assert!(!column_can_resize(options, &col));
    }
}
