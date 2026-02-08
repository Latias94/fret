use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
    contains_ascii_case_insensitive,
};
use serde::Deserialize;

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
struct RowSelectionDetail {
    is_selected: BTreeMap<String, bool>,
    is_some_selected: BTreeMap<String, bool>,
    is_all_sub_rows_selected: BTreeMap<String, bool>,
    can_select: BTreeMap<String, bool>,
    can_multi_select: BTreeMap<String, bool>,
    can_select_sub_rows: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    #[allow(dead_code)]
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    #[serde(default)]
    selected: Option<RowModelSnapshot>,
    #[serde(default)]
    filtered_selected: Option<RowModelSnapshot>,
    #[serde(default)]
    grouped_selected: Option<RowModelSnapshot>,
    #[serde(default)]
    row_selection_detail: Option<RowSelectionDetail>,
    #[serde(default)]
    is_all_rows_selected: Option<bool>,
    #[serde(default)]
    is_some_rows_selected: Option<bool>,
    #[serde(default)]
    is_all_page_rows_selected: Option<bool>,
    #[serde(default)]
    is_some_page_rows_selected: Option<bool>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "toggleRowSelected")]
    ToggleRowSelected {
        row_id: String,
        #[serde(default)]
        value: Option<bool>,
        #[serde(default = "default_true")]
        select_children: bool,
    },
}

fn default_true() -> bool {
    true
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
fn tanstack_v8_selection_tree_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("selection_tree.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "selection_tree");

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

    for snap in fixture.snapshots {
        let on_row_selection_change_mode = snap
            .options
            .get("__onRowSelectionChange")
            .and_then(|v| v.as_str());

        let enable_row_selection_mode = snap
            .options
            .get("__enableRowSelection")
            .and_then(|v| v.as_str());
        let enable_multi_row_selection_mode = snap
            .options
            .get("__enableMultiRowSelection")
            .and_then(|v| v.as_str());
        let enable_sub_row_selection_mode = snap
            .options
            .get("__enableSubRowSelection")
            .and_then(|v| v.as_str());

        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        for action in &snap.actions {
            let mut builder = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .get_sub_rows(|row, _idx| {
                    if row.sub_rows.is_empty() {
                        None
                    } else {
                        Some(row.sub_rows.as_slice())
                    }
                })
                .state(state.clone())
                .options(options);

            if enable_row_selection_mode == Some("odd_ids") {
                builder = builder.enable_row_selection_by(|row_key, _row| row_key.0 % 2 == 1);
            } else if enable_row_selection_mode == Some("except_11") {
                builder = builder.enable_row_selection_by(|row_key, _row| row_key.0 != 11);
            } else if enable_row_selection_mode == Some("always_false") {
                builder = builder.enable_row_selection_by(|_row_key, _row| false);
            }

            if enable_multi_row_selection_mode == Some("always_false") {
                builder = builder.enable_multi_row_selection_by(|_row_key, _row| false);
            }

            if enable_sub_row_selection_mode == Some("disable_root_1") {
                builder = builder.enable_sub_row_selection_by(|row_key, _row| row_key.0 != 1);
            } else if enable_sub_row_selection_mode == Some("always_false") {
                builder = builder.enable_sub_row_selection_by(|_row_key, _row| false);
            }

            let table = builder.build();

            if on_row_selection_change_mode == Some("noop") {
                continue;
            }

            match action {
                FixtureAction::ToggleRowSelected {
                    row_id,
                    value,
                    select_children,
                } => {
                    let row_key = RowKey(
                        row_id
                            .parse::<u64>()
                            .unwrap_or_else(|_| panic!("invalid row_id: {row_id}")),
                    );
                    state.row_selection =
                        table.toggled_row_selected(row_key, *value, *select_children);
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
                state.row_selection, expected_state.row_selection,
                "snapshot {} next_state.rowSelection mismatch",
                snap.id
            );
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .get_sub_rows(|row, _idx| {
                if row.sub_rows.is_empty() {
                    None
                } else {
                    Some(row.sub_rows.as_slice())
                }
            })
            .state(state)
            .options(options);

        let mut builder = table;

        if enable_row_selection_mode == Some("odd_ids") {
            builder = builder.enable_row_selection_by(|row_key, _row| row_key.0 % 2 == 1);
        } else if enable_row_selection_mode == Some("except_11") {
            builder = builder.enable_row_selection_by(|row_key, _row| row_key.0 != 11);
        } else if enable_row_selection_mode == Some("always_false") {
            builder = builder.enable_row_selection_by(|_row_key, _row| false);
        }

        if enable_multi_row_selection_mode == Some("always_false") {
            builder = builder.enable_multi_row_selection_by(|_row_key, _row| false);
        }

        if enable_sub_row_selection_mode == Some("disable_root_1") {
            builder = builder.enable_sub_row_selection_by(|row_key, _row| row_key.0 != 1);
        } else if enable_sub_row_selection_mode == Some("always_false") {
            builder = builder.enable_sub_row_selection_by(|_row_key, _row| false);
        }

        let table = builder.build();

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
            paginated.root, snap.expect.row_model.root,
            "snapshot {} row_model root mismatch",
            snap.id
        );
        assert_eq!(
            paginated.flat, snap.expect.row_model.flat,
            "snapshot {} row_model flat mismatch",
            snap.id
        );

        if let Some(expected) = snap.expect.selected.as_ref() {
            let selected = snapshot_row_model(table.selected_row_model());
            assert_eq!(
                selected.root, expected.root,
                "snapshot {} selected root mismatch",
                snap.id
            );
            assert_eq!(
                selected.flat, expected.flat,
                "snapshot {} selected flat mismatch",
                snap.id
            );
        }
        if let Some(expected) = snap.expect.filtered_selected.as_ref() {
            let selected = snapshot_row_model(table.filtered_selected_row_model());
            assert_eq!(
                selected.root, expected.root,
                "snapshot {} filtered_selected root mismatch",
                snap.id
            );
            assert_eq!(
                selected.flat, expected.flat,
                "snapshot {} filtered_selected flat mismatch",
                snap.id
            );
        }
        if let Some(expected) = snap.expect.grouped_selected.as_ref() {
            let selected = snapshot_row_model(table.grouped_selected_row_model());
            assert_eq!(
                selected.root, expected.root,
                "snapshot {} grouped_selected root mismatch",
                snap.id
            );
            assert_eq!(
                selected.flat, expected.flat,
                "snapshot {} grouped_selected flat mismatch",
                snap.id
            );
        }

        if let Some(expected) = snap.expect.row_selection_detail.as_ref() {
            for (row_id, expected_value) in &expected.is_selected {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_is_selected(row_key),
                    *expected_value,
                    "snapshot {} is_selected[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
            for (row_id, expected_value) in &expected.is_some_selected {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_is_some_selected(row_key),
                    *expected_value,
                    "snapshot {} is_some_selected[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
            for (row_id, expected_value) in &expected.is_all_sub_rows_selected {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_is_all_sub_rows_selected(row_key),
                    *expected_value,
                    "snapshot {} is_all_sub_rows_selected[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
            for (row_id, expected_value) in &expected.can_select {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_can_select(row_key),
                    *expected_value,
                    "snapshot {} can_select[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
            for (row_id, expected_value) in &expected.can_multi_select {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_can_multi_select(row_key),
                    *expected_value,
                    "snapshot {} can_multi_select[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
            for (row_id, expected_value) in &expected.can_select_sub_rows {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                assert_eq!(
                    table.row_can_select_sub_rows(row_key),
                    *expected_value,
                    "snapshot {} can_select_sub_rows[{}] mismatch",
                    snap.id,
                    row_id
                );
            }
        }

        if let Some(expected) = snap.expect.is_all_rows_selected {
            assert_eq!(
                table.is_all_rows_selected(),
                expected,
                "snapshot {} is_all_rows_selected mismatch",
                snap.id
            );
        }
        if let Some(expected) = snap.expect.is_some_rows_selected {
            assert_eq!(
                table.is_some_rows_selected(),
                expected,
                "snapshot {} is_some_rows_selected mismatch",
                snap.id
            );
        }
        if let Some(expected) = snap.expect.is_all_page_rows_selected {
            assert_eq!(
                table.is_all_page_rows_selected(),
                expected,
                "snapshot {} is_all_page_rows_selected mismatch",
                snap.id
            );
        }
        if let Some(expected) = snap.expect.is_some_page_rows_selected {
            assert_eq!(
                table.is_some_page_rows_selected(),
                expected,
                "snapshot {} is_some_page_rows_selected mismatch",
                snap.id
            );
        }
    }
}
