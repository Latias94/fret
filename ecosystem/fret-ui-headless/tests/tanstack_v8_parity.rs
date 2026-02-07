use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState, TanStackValue,
    contains_ascii_case_insensitive, sort_for_column,
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
    sorting_helpers: Option<SortingHelpersSnapshot>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct SortingHelpersSnapshot {
    columns: std::collections::BTreeMap<String, SortingHelperColumn>,
}

#[derive(Debug, Clone, Deserialize)]
struct SortingHelperColumn {
    can_sort: bool,
    can_multi_sort: bool,
    is_sorted: Option<String>,
    sort_index: i32,
    auto_sort_dir: Option<String>,
    first_sort_dir: Option<String>,
    next_sorting_order: Option<String>,
    next_sorting_order_multi: Option<String>,
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
    #[serde(rename = "toggleSortingHandler")]
    ToggleSortingHandler {
        column_id: String,
        #[serde(default)]
        event_multi: bool,
    },
    #[serde(rename = "clearSorting")]
    ClearSorting { column_id: String },
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
        .filter_map(|&i| model.row(i).map(|r| r.key.0.to_string()))
        .collect();
    let flat = model
        .flat_rows()
        .iter()
        .filter_map(|&i| model.row(i).map(|r| r.key.0.to_string()))
        .collect();
    RowModelSnapshot { root, flat }
}

#[test]
fn tanstack_v8_demo_process_parity_core_filter_sort_paginate() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("demo_process.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "demo_process");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.name.cmp(&b.name))
            .sort_value_by(|row: &FixtureRow| {
                TanStackValue::String(Arc::<str>::from(row.name.clone()))
            })
            .filter_by(|row: &FixtureRow, needle: &str| {
                contains_ascii_case_insensitive(&row.name, needle)
            }),
        ColumnDef::<FixtureRow>::new("status")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.status.cmp(&b.status))
            .sort_value_by(|row: &FixtureRow| {
                TanStackValue::String(Arc::<str>::from(row.status.clone()))
            })
            .filter_by(|row: &FixtureRow, needle: &str| {
                contains_ascii_case_insensitive(&row.status, needle)
            }),
        ColumnDef::<FixtureRow>::new("cpu")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
            .filter_by(|row: &FixtureRow, needle: &str| {
                let Ok(v) = needle.parse::<u64>() else {
                    return false;
                };
                row.cpu == v
            }),
        ColumnDef::<FixtureRow>::new("cpu_desc_first")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
            .sort_desc_first(true),
        ColumnDef::<FixtureRow>::new("cpu_no_multi")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
            .enable_multi_sort(false),
        ColumnDef::<FixtureRow>::new("cpu_no_sort")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
            .enable_sorting(false),
        ColumnDef::<FixtureRow>::new("cpu_invert")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
            .invert_sorting(true),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.mem_mb.cmp(&b.mem_mb))
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.mem_mb as f64))
            .filter_by(|row: &FixtureRow, needle: &str| {
                let Ok(v) = needle.parse::<u64>() else {
                    return false;
                };
                row.mem_mb == v
            }),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        if !snap.actions.is_empty() {
            for action in &snap.actions {
                let table_for_action = Table::builder(&data)
                    .columns(columns.clone())
                    .get_row_key(|row, _idx, _parent| RowKey(row.id))
                    .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                    .state(state.clone())
                    .options(options)
                    .build();

                match action {
                    FixtureAction::ToggleSorting { column_id, multi } => {
                        state.sorting = table_for_action
                            .toggled_column_sorting_tanstack(column_id.as_str(), *multi, false)
                            .unwrap_or_else(|| panic!("unknown action column_id: {column_id}"));
                    }
                    FixtureAction::ToggleSortingHandler {
                        column_id,
                        event_multi,
                    } => {
                        state.sorting = table_for_action
                            .toggled_column_sorting_handler_tanstack(
                                column_id.as_str(),
                                *event_multi,
                                false,
                            )
                            .unwrap_or_else(|| panic!("unknown action column_id: {column_id}"));
                    }
                    FixtureAction::ClearSorting { column_id } => {
                        state.sorting = table_for_action
                            .cleared_column_sorting(column_id.as_str())
                            .unwrap_or_else(|| panic!("unknown action column_id: {column_id}"));
                    }
                }
            }

            if let Some(expected_next) = snap.expect.next_state.as_ref() {
                let tanstack_next =
                    TanStackTableState::from_json(expected_next).expect("tanstack next_state");
                let expected_state = tanstack_next
                    .to_table_state()
                    .expect("next_state conversion");
                assert_eq!(
                    state.sorting, expected_state.sorting,
                    "snapshot {} next_state.sorting mismatch",
                    snap.id
                );
            }
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
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

        if let Some(expected) = snap.expect.sorting_helpers.as_ref() {
            fn dir_for_desc(desc: bool) -> &'static str {
                if desc { "desc" } else { "asc" }
            }

            fn opt_dir_for_desc(desc: Option<bool>) -> Option<&'static str> {
                desc.map(dir_for_desc)
            }

            fn next_dir_for_desc(next: Option<Option<bool>>) -> Option<&'static str> {
                next.and_then(|v| v).map(dir_for_desc)
            }

            for (col_id, exp) in &expected.columns {
                assert_eq!(
                    table.column_can_sort(col_id.as_str()),
                    Some(exp.can_sort),
                    "snapshot {} sorting_helpers.columns[{}].can_sort mismatch",
                    snap.id,
                    col_id
                );

                assert_eq!(
                    table.column_is_sorted(col_id.as_str()),
                    Some(exp.is_sorted.is_some()),
                    "snapshot {} sorting_helpers.columns[{}].is_sorted mismatch",
                    snap.id,
                    col_id
                );

                assert_eq!(
                    table.column_sort_index(col_id.as_str()),
                    Some(exp.sort_index),
                    "snapshot {} sorting_helpers.columns[{}].sort_index mismatch",
                    snap.id,
                    col_id
                );

                assert_eq!(
                    table.column_can_multi_sort(col_id.as_str()),
                    Some(exp.can_multi_sort),
                    "snapshot {} sorting_helpers.columns[{}].can_multi_sort mismatch",
                    snap.id,
                    col_id
                );

                let expected_desc = exp.is_sorted.as_deref().map(|s| s == "desc");
                assert_eq!(
                    sort_for_column(&table.state().sorting, col_id.as_str()),
                    expected_desc,
                    "snapshot {} sorting_helpers.columns[{}].sort_for_column mismatch",
                    snap.id,
                    col_id
                );

                assert_eq!(
                    opt_dir_for_desc(table.column_auto_sort_dir_desc_tanstack(col_id.as_str())),
                    exp.auto_sort_dir.as_deref(),
                    "snapshot {} sorting_helpers.columns[{}].auto_sort_dir mismatch",
                    snap.id,
                    col_id
                );
                assert_eq!(
                    opt_dir_for_desc(table.column_first_sort_dir_desc_tanstack(col_id.as_str())),
                    exp.first_sort_dir.as_deref(),
                    "snapshot {} sorting_helpers.columns[{}].first_sort_dir mismatch",
                    snap.id,
                    col_id
                );
                assert_eq!(
                    next_dir_for_desc(
                        table.column_next_sorting_order_desc_tanstack(col_id.as_str(), false),
                    ),
                    exp.next_sorting_order.as_deref(),
                    "snapshot {} sorting_helpers.columns[{}].next_sorting_order mismatch",
                    snap.id,
                    col_id
                );
                assert_eq!(
                    next_dir_for_desc(
                        table.column_next_sorting_order_desc_tanstack(col_id.as_str(), true),
                    ),
                    exp.next_sorting_order_multi.as_deref(),
                    "snapshot {} sorting_helpers.columns[{}].next_sorting_order_multi mismatch",
                    snap.id,
                    col_id
                );
            }
        }

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

        // Also gate the explicit pagination model against the fixture's `paginated` snapshot.
        // Today we treat `paginate_expanded_rows=true` as the default, so `row_model` matches the
        // paginated model when expansion is inactive.
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
    }
}
