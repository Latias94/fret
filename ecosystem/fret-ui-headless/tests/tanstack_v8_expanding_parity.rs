use std::collections::HashMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowKey, Table, TanStackTableOptions, TanStackTableState,
    contains_ascii_case_insensitive,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
    #[serde(default, rename = "subRows")]
    sub_rows: Vec<FixtureRow>,
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
    expanded: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    #[serde(default)]
    is_all_rows_expanded: bool,
    #[serde(default)]
    is_some_rows_expanded: bool,
    #[serde(default)]
    can_some_rows_expand: bool,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "toggleRowExpanded")]
    ToggleRowExpanded {
        row_id: String,
        #[serde(default)]
        value: Option<bool>,
    },
    #[serde(rename = "toggleAllRowsExpanded")]
    ToggleAllRowsExpanded {
        #[serde(default)]
        value: Option<bool>,
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

fn option_marker<'a>(options: &'a Value, key: &str) -> Option<&'a str> {
    options.get(key)?.as_str()
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
fn tanstack_v8_expanding_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("expanding.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "expanding");

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
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu)),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.mem_mb.cmp(&b.mem_mb)),
    ];

    let _column_by_id: HashMap<&str, &ColumnDef<FixtureRow>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    for snap in fixture.snapshots {
        let get_row_can_expand_mode = option_marker(&snap.options, "__getRowCanExpand");
        let get_is_row_expanded_mode = option_marker(&snap.options, "__getIsRowExpanded");
        let on_expanded_change_mode = option_marker(&snap.options, "__onExpandedChange");
        let get_expanded_row_model_mode = option_marker(&snap.options, "__getExpandedRowModel");

        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        for action in &snap.actions {
            let mut builder = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_sub_rows(|row, _idx| {
                    if row.sub_rows.is_empty() {
                        None
                    } else {
                        Some(row.sub_rows.as_slice())
                    }
                });
            if get_row_can_expand_mode == Some("only_root_1") {
                builder = builder.get_row_can_expand_by(|row_key, _row| row_key.0 == 1);
            }
            if get_is_row_expanded_mode == Some("always_false") {
                builder = builder.get_is_row_expanded_by(|_row_key, _row| false);
            }
            if get_expanded_row_model_mode == Some("pre_expanded") {
                builder = builder.override_expanded_row_model_pre_expanded();
            }
            let table = builder.state(state.clone()).options(options).build();

            match action {
                FixtureAction::ToggleRowExpanded { row_id, value } => {
                    let row_key = RowKey(
                        row_id
                            .parse::<u64>()
                            .unwrap_or_else(|_| panic!("invalid row_id: {row_id}")),
                    );
                    if on_expanded_change_mode != Some("noop") {
                        state.expanding = table.toggled_row_expanded(row_key, *value);
                    }
                }
                FixtureAction::ToggleAllRowsExpanded { value } => {
                    if on_expanded_change_mode != Some("noop") {
                        state.expanding = table.toggled_all_rows_expanded(*value);
                    }
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
                state.expanding, expected_state.expanding,
                "snapshot {} next_state.expanded mismatch",
                snap.id
            );
        }

        let mut builder = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_sub_rows(|row, _idx| {
                if row.sub_rows.is_empty() {
                    None
                } else {
                    Some(row.sub_rows.as_slice())
                }
            });
        if get_row_can_expand_mode == Some("only_root_1") {
            builder = builder.get_row_can_expand_by(|row_key, _row| row_key.0 == 1);
        }
        if get_is_row_expanded_mode == Some("always_false") {
            builder = builder.get_is_row_expanded_by(|_row_key, _row| false);
        }
        if get_expanded_row_model_mode == Some("pre_expanded") {
            builder = builder.override_expanded_row_model_pre_expanded();
        }
        let table = builder.state(state).options(options).build();

        let core = snapshot_row_model(table.core_row_model());
        let filtered = snapshot_row_model(table.filtered_row_model());
        let sorted = snapshot_row_model(table.sorted_row_model());
        let expanded = snapshot_row_model(table.expanded_row_model());
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
            expanded.root, snap.expect.expanded.root,
            "snapshot {} expanded root mismatch",
            snap.id
        );
        assert_eq!(
            expanded.flat, snap.expect.expanded.flat,
            "snapshot {} expanded flat mismatch",
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

        assert_eq!(
            table.is_all_rows_expanded(),
            snap.expect.is_all_rows_expanded,
            "snapshot {} is_all_rows_expanded mismatch",
            snap.id
        );
        assert_eq!(
            table.is_some_rows_expanded(),
            snap.expect.is_some_rows_expanded,
            "snapshot {} is_some_rows_expanded mismatch",
            snap.id
        );
        assert_eq!(
            table.can_some_rows_expand(),
            snap.expect.can_some_rows_expand,
            "snapshot {} can_some_rows_expand mismatch",
            snap.id
        );
    }
}
