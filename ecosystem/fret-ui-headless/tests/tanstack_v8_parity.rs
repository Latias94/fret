use std::collections::HashMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
    contains_ascii_case_insensitive, toggle_sorting_handler_tanstack, toggle_sorting_tanstack,
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
    #[serde(rename = "toggleSortingHandler")]
    ToggleSortingHandler {
        column_id: String,
        #[serde(default)]
        event_multi: bool,
    },
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
            .filter_by(|row: &FixtureRow, needle: &str| {
                contains_ascii_case_insensitive(&row.name, needle)
            }),
        ColumnDef::<FixtureRow>::new("status")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.status.cmp(&b.status))
            .filter_by(|row: &FixtureRow, needle: &str| {
                contains_ascii_case_insensitive(&row.status, needle)
            }),
        ColumnDef::<FixtureRow>::new("cpu")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .filter_by(|row: &FixtureRow, needle: &str| {
                let Ok(v) = needle.parse::<u64>() else {
                    return false;
                };
                row.cpu == v
            }),
        ColumnDef::<FixtureRow>::new("cpu_desc_first")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_desc_first(true),
        ColumnDef::<FixtureRow>::new("cpu_no_multi")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .enable_multi_sort(false),
        ColumnDef::<FixtureRow>::new("cpu_no_sort")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .enable_sorting(false),
        ColumnDef::<FixtureRow>::new("cpu_invert")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .invert_sorting(true),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.mem_mb.cmp(&b.mem_mb))
            .filter_by(|row: &FixtureRow, needle: &str| {
                let Ok(v) = needle.parse::<u64>() else {
                    return false;
                };
                row.mem_mb == v
            }),
    ];

    let column_by_id: HashMap<&str, &ColumnDef<FixtureRow>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        if !snap.actions.is_empty() {
            for action in &snap.actions {
                match action {
                    FixtureAction::ToggleSorting { column_id, multi } => {
                        let Some(column) = column_by_id.get(column_id.as_str()).copied() else {
                            panic!("unknown action column_id: {}", column_id);
                        };
                        toggle_sorting_tanstack(&mut state.sorting, column, options, *multi, false);
                    }
                    FixtureAction::ToggleSortingHandler {
                        column_id,
                        event_multi,
                    } => {
                        let Some(column) = column_by_id.get(column_id.as_str()).copied() else {
                            panic!("unknown action column_id: {}", column_id);
                        };
                        toggle_sorting_handler_tanstack(
                            &mut state.sorting,
                            column,
                            options,
                            *event_multi,
                            false,
                        );
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
