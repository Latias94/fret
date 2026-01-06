use std::collections::HashMap;

use super::ColumnId;

/// TanStack-compatible column sizing map: `column_id -> size`.
pub type ColumnSizingState = HashMap<ColumnId, f32>;

pub fn column_size(state: &ColumnSizingState, column: &ColumnId) -> Option<f32> {
    state.get(column).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_size_reads_from_map() {
        let mut state = ColumnSizingState::default();
        state.insert(ColumnId::from("a"), 123.0);

        assert_eq!(column_size(&state, &ColumnId::from("a")), Some(123.0));
        assert_eq!(column_size(&state, &ColumnId::from("b")), None);
    }
}
