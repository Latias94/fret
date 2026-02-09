use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, FilteringFnSpec, RowId, RowKey, Table, TableState, TanStackAutoResetQueue,
    TanStackTableOptions, TanStackTableState, TanStackValue,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "toggleSorting")]
    ToggleSorting {
        column_id: String,
        #[serde(default)]
        multi: bool,
    },
    #[serde(rename = "setColumnFilterValue")]
    SetColumnFilterValue {
        column_id: String,
        value: serde_json::Value,
    },
    #[serde(rename = "setGlobalFilterValue")]
    SetGlobalFilterValue { value: serde_json::Value },
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    #[serde(default)]
    state: serde_json::Value,
    #[serde(default)]
    actions: Vec<FixtureAction>,
    expect: FixtureExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

fn snapshot_row_model<'a, TData>(
    model: &fret_ui_headless::table::RowModel<'a, TData>,
) -> RowModelSnapshot {
    let root = model
        .root_rows()
        .iter()
        .filter_map(|&i| model.row(i).map(|r| r.id.as_str().to_string()))
        .collect();
    let flat = model
        .flat_rows()
        .iter()
        .filter_map(|&i| model.row(i).map(|r| r.id.as_str().to_string()))
        .collect();
    RowModelSnapshot { root, flat }
}

fn tanstack_value_str(s: &str) -> TanStackValue {
    TanStackValue::String(Arc::<str>::from(s))
}

fn tanstack_value_num(n: u64) -> TanStackValue {
    TanStackValue::Number(n as f64)
}

#[test]
fn tanstack_v8_auto_reset_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("auto_reset.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "auto_reset");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name")
            .sort_value_by(|row: &FixtureRow| tanstack_value_str(&row.name))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("status")
            .sort_value_by(|row: &FixtureRow| tanstack_value_str(&row.status))
            .facet_str_by(|row: &FixtureRow| row.status.as_str())
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("cpu")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("cpu_desc_first")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto()
            .sort_desc_first(true),
        ColumnDef::<FixtureRow>::new("cpu_no_multi")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto()
            .enable_multi_sort(false),
        ColumnDef::<FixtureRow>::new("cpu_no_sort")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto()
            .enable_sorting(false),
        ColumnDef::<FixtureRow>::new("cpu_invert")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto()
            .invert_sorting(true),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.mem_mb))
            .sorting_fn_auto()
            .filtering_fn_auto(),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        // TanStack: table-level reset APIs target `options.initialState`, not `options.state`.
        // Auto reset behaviors (e.g. `autoResetPageIndex`) also reset to `initialState`.
        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        // Simulate TanStack `_queue` behavior: register-first, then coalesced resets per render pass.
        // The fixture generator performs an initial `getRowModel()` call before actions.
        let mut auto_reset = TanStackAutoResetQueue::default();
        {
            let table_initial = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options)
                .global_filter_fn(FilteringFnSpec::Auto)
                .build();

            auto_reset.begin_render_pass();
            auto_reset.auto_reset_page_index(&table_initial);
            auto_reset.flush(&table_initial, &mut state);
        }

        for action in &snap.actions {
            match action {
                FixtureAction::ToggleSorting { column_id, multi } => {
                    let table_for_action = Table::builder(&data)
                        .columns(columns.clone())
                        .get_row_key(|row, _idx, _parent| RowKey(row.id))
                        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                        .initial_state(initial_state.clone())
                        .state(state.clone())
                        .options(options)
                        .global_filter_fn(FilteringFnSpec::Auto)
                        .build();

                    state.sorting = table_for_action
                        .toggled_column_sorting_tanstack(column_id.as_str(), *multi, false)
                        .unwrap_or_else(|| panic!("unknown action column_id: {column_id}"));
                }
                FixtureAction::SetColumnFilterValue { column_id, value } => {
                    let table_for_action = Table::builder(&data)
                        .columns(columns.clone())
                        .get_row_key(|row, _idx, _parent| RowKey(row.id))
                        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                        .initial_state(initial_state.clone())
                        .state(state.clone())
                        .options(options)
                        .global_filter_fn(FilteringFnSpec::Auto)
                        .build();

                    let updater = table_for_action
                        .column_filters_updater_set_value(column_id.as_str(), value.clone())
                        .unwrap_or_else(|| panic!("unknown action column_id: {column_id}"));
                    state.column_filters = updater.apply(&state.column_filters);
                }
                FixtureAction::SetGlobalFilterValue { value } => {
                    let table_for_action = Table::builder(&data)
                        .columns(columns.clone())
                        .get_row_key(|row, _idx, _parent| RowKey(row.id))
                        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                        .initial_state(initial_state.clone())
                        .state(state.clone())
                        .options(options)
                        .global_filter_fn(FilteringFnSpec::Auto)
                        .build();
                    let updater =
                        table_for_action.global_filter_updater_set_value(Some(value.clone()));
                    state.global_filter = updater.apply(&state.global_filter);
                }
            }

            let table_post = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options)
                .global_filter_fn(FilteringFnSpec::Auto)
                .build();

            // TanStack: multiple derived-model memos may queue `_autoReset*()` within a single render
            // pass. We call them multiple times and rely on the queue to coalesce.
            auto_reset.begin_render_pass();
            auto_reset.auto_reset_page_index(&table_post);
            auto_reset.auto_reset_page_index(&table_post);
            auto_reset.auto_reset_page_index(&table_post);
            auto_reset.flush(&table_post, &mut state);
        }

        if let Some(expected_next) = snap.expect.next_state.as_ref() {
            let tanstack_next =
                TanStackTableState::from_json(expected_next).expect("tanstack next_state");
            let expected_state = tanstack_next
                .to_table_state()
                .expect("next_state conversion");
            assert_eq!(
                state.pagination, expected_state.pagination,
                "snapshot {} next_state.pagination mismatch",
                snap.id
            );
            assert_eq!(
                state.sorting, expected_state.sorting,
                "snapshot {} next_state.sorting mismatch",
                snap.id
            );
            assert_eq!(
                state.global_filter, expected_state.global_filter,
                "snapshot {} next_state.global_filter mismatch",
                snap.id
            );
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .global_filter_fn(FilteringFnSpec::Auto)
            .build();

        let core = snapshot_row_model(table.core_row_model());
        let filtered = snapshot_row_model(table.filtered_row_model());
        let sorted = snapshot_row_model(table.sorted_row_model());
        let paginated = snapshot_row_model(table.row_model());

        assert_eq!(
            core.root, snap.expect.core.root,
            "snapshot {} core root mismatch",
            snap.id
        );
        assert_eq!(
            core.flat, snap.expect.core.flat,
            "snapshot {} core flat mismatch",
            snap.id
        );
        assert_eq!(
            filtered.root, snap.expect.filtered.root,
            "snapshot {} filtered root mismatch",
            snap.id
        );
        assert_eq!(
            filtered.flat, snap.expect.filtered.flat,
            "snapshot {} filtered flat mismatch",
            snap.id
        );
        assert_eq!(
            sorted.root, snap.expect.sorted.root,
            "snapshot {} sorted root mismatch",
            snap.id
        );
        assert_eq!(
            sorted.flat, snap.expect.sorted.flat,
            "snapshot {} sorted flat mismatch",
            snap.id
        );
        assert_eq!(
            paginated.root, snap.expect.paginated.root,
            "snapshot {} paginated root mismatch",
            snap.id
        );
        assert_eq!(
            paginated.flat, snap.expect.paginated.flat,
            "snapshot {} paginated flat mismatch",
            snap.id
        );

        // Our engine's `row_model()` corresponds to TanStack's `getRowModel()` (post-pagination).
        assert_eq!(
            paginated.root, snap.expect.row_model.root,
            "snapshot {} row_model root mismatch",
            snap.id
        );
        assert_eq!(
            paginated.flat, snap.expect.row_model.flat,
            "snapshot {} row_model flat mismatch",
            snap.id
        );
    }
}
