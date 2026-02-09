//! TanStack Table v8 auto-reset scheduling helpers.
//!
//! TanStack's engine uses a per-table `_queue` microtask scheduler to coalesce multiple
//! `_autoReset*` calls within a single render pass (derived model recompute).
//!
//! Fret's headless table is "pure" (state is owned externally), so we expose an explicit queue
//! that callers can keep outside ephemeral table instances (including rebuild-each-frame setups).

use super::{Table, TableState};

/// Explicit auto-reset queue that models TanStack's `_queue` coalescing behavior.
///
/// Usage pattern:
/// - Keep this queue outside ephemeral `Table` instances.
/// - At the start of a render pass, call `begin_render_pass()`.
/// - When a derived row model is recomputed and would call TanStack `_autoReset*`, call the
///   corresponding `auto_reset_*` method (you may call it multiple times; it will coalesce).
/// - At the end of the pass, call `flush(...)` to apply any pending resets.
#[derive(Debug, Default, Clone)]
pub struct TanStackAutoResetQueue {
    page_index_registered: bool,
    expanded_registered: bool,
    pending_page_index_register: bool,
    pending_expanded_register: bool,
    pending_page_index_reset: bool,
    pending_expanded_reset: bool,
}

impl TanStackAutoResetQueue {
    /// Starts a new "render pass" (logical tick). Pending resets are cleared, registration is kept.
    pub fn begin_render_pass(&mut self) {
        self.pending_page_index_register = false;
        self.pending_expanded_register = false;
        self.pending_page_index_reset = false;
        self.pending_expanded_reset = false;
    }

    /// TanStack-aligned: queues `_autoResetPageIndex()` (register-first, coalesced).
    pub fn auto_reset_page_index<TData>(&mut self, table: &Table<'_, TData>) {
        if !self.page_index_registered {
            // TanStack sets `registered=true` via `_queue` (microtask), so multiple calls in the
            // same render pass still count as "first register" and should not schedule a reset.
            self.pending_page_index_register = true;
            return;
        }
        if table.should_auto_reset_page_index() {
            self.pending_page_index_reset = true;
        }
    }

    /// TanStack-aligned: queues `_autoResetExpanded()` (register-first, coalesced).
    pub fn auto_reset_expanded<TData>(&mut self, table: &Table<'_, TData>) {
        if !self.expanded_registered {
            self.pending_expanded_register = true;
            return;
        }
        if table.should_auto_reset_expanded() {
            self.pending_expanded_reset = true;
        }
    }

    /// Applies any pending auto resets to `state` (at most once per kind per render pass).
    ///
    /// Notes:
    /// - TanStack resets to `initialState`, not the feature default state, so we use
    ///   `reset_*(default_state=false)`.
    pub fn flush<TData>(&mut self, table: &Table<'_, TData>, state: &mut TableState) {
        if self.pending_expanded_reset {
            state.expanding = table.reset_expanded(false);
        }
        if self.pending_page_index_reset {
            state.pagination = table.reset_page_index(false);
        }
        if self.pending_expanded_register {
            self.expanded_registered = true;
        }
        if self.pending_page_index_register {
            self.page_index_registered = true;
        }
        self.pending_expanded_register = false;
        self.pending_page_index_register = false;
        self.pending_expanded_reset = false;
        self.pending_page_index_reset = false;
    }

    pub fn expanded_registered(&self) -> bool {
        self.expanded_registered
    }

    pub fn page_index_registered(&self) -> bool {
        self.page_index_registered
    }

    pub fn pending_expanded_reset(&self) -> bool {
        self.pending_expanded_reset
    }

    pub fn pending_page_index_reset(&self) -> bool {
        self.pending_page_index_reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{
        ColumnDef, ExpandingState, FilteringFnSpec, PaginationState, RowId, RowKey,
    };

    #[derive(Debug, Clone)]
    struct Row {
        id: u64,
        cpu: u64,
    }

    fn build_table(
        data: &[Row],
        state: TableState,
        initial: TableState,
        manual_pagination: bool,
    ) -> Table<'_, Row> {
        let columns: Vec<ColumnDef<Row>> = vec![
            ColumnDef::<Row>::new("cpu")
                .sort_value_by(|r: &Row| crate::table::TanStackValue::Number(r.cpu as f64))
                .sorting_fn_auto()
                .filtering_fn_auto(),
        ];

        Table::builder(data)
            .columns(columns)
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .initial_state(initial)
            .options(crate::table::TableOptions {
                manual_pagination,
                ..Default::default()
            })
            .global_filter_fn(FilteringFnSpec::Auto)
            .build()
    }

    #[test]
    fn page_index_register_first_then_reset_coalesces() {
        let data = vec![Row { id: 1, cpu: 10 }, Row { id: 2, cpu: 20 }];

        let mut state = TableState::default();
        state.pagination = PaginationState {
            page_index: 1,
            page_size: 2,
        };

        let initial = TableState::default();

        let table = build_table(&data, state.clone(), initial.clone(), false);

        let mut q = TanStackAutoResetQueue::default();

        // First pass registers; no reset even if we call it multiple times.
        q.begin_render_pass();
        q.auto_reset_page_index(&table);
        q.auto_reset_page_index(&table);
        q.flush(&table, &mut state);
        assert!(q.page_index_registered());
        assert_eq!(state.pagination.page_index, 1);

        // Second pass schedules reset; multiple calls are coalesced.
        q.begin_render_pass();
        q.auto_reset_page_index(&table);
        q.auto_reset_page_index(&table);
        q.auto_reset_page_index(&table);
        assert!(q.pending_page_index_reset());
        q.flush(&table, &mut state);
        assert_eq!(state.pagination.page_index, initial.pagination.page_index);
        assert!(!q.pending_page_index_reset());
    }

    #[test]
    fn manual_pagination_disables_auto_reset_by_default() {
        let data = vec![Row { id: 1, cpu: 10 }, Row { id: 2, cpu: 20 }];

        let mut state = TableState::default();
        state.pagination.page_index = 3;

        let initial = TableState::default();
        let table = build_table(&data, state.clone(), initial.clone(), true);

        let mut q = TanStackAutoResetQueue::default();

        // Register.
        q.begin_render_pass();
        q.auto_reset_page_index(&table);
        q.flush(&table, &mut state);
        assert_eq!(state.pagination.page_index, 3);

        // Should not schedule because `manualPagination=true` and no override.
        q.begin_render_pass();
        q.auto_reset_page_index(&table);
        q.flush(&table, &mut state);
        assert_eq!(state.pagination.page_index, 3);
    }

    #[test]
    fn expanded_register_first_then_reset() {
        let data = vec![Row { id: 1, cpu: 10 }, Row { id: 2, cpu: 20 }];

        let mut state = TableState::default();
        state.expanding = ExpandingState::All;

        let mut initial = TableState::default();
        initial.expanding = ExpandingState::default();

        let table = build_table(&data, state.clone(), initial.clone(), false);

        let mut q = TanStackAutoResetQueue::default();

        q.begin_render_pass();
        q.auto_reset_expanded(&table);
        q.flush(&table, &mut state);
        assert_eq!(state.expanding, ExpandingState::All);

        q.begin_render_pass();
        q.auto_reset_expanded(&table);
        q.flush(&table, &mut state);
        assert_eq!(state.expanding, initial.expanding);
    }
}
