use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowKey, Table, TableState, TanStackTableOptions, TanStackTableState,
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
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "resetSorting")]
    ResetSorting {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetColumnFilters")]
    ResetColumnFilters {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetGlobalFilter")]
    ResetGlobalFilter {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetGrouping")]
    ResetGrouping {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetColumnVisibility")]
    ResetColumnVisibility {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetColumnOrder")]
    ResetColumnOrder {
        #[serde(default)]
        default_state: Option<bool>,
    },
    #[serde(rename = "resetRowSelection")]
    ResetRowSelection {
        #[serde(default)]
        default_state: Option<bool>,
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

#[test]
fn tanstack_v8_resets_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("resets.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "resets");

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
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        // TanStack: reset APIs target `options.initialState`, not `options.state`.
        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::ResetSorting { default_state } => {
                    state.sorting = table.reset_sorting(default_state.unwrap_or(false));
                }
                FixtureAction::ResetColumnFilters { default_state } => {
                    state.column_filters =
                        table.reset_column_filters(default_state.unwrap_or(false));
                }
                FixtureAction::ResetGlobalFilter { default_state } => {
                    state.global_filter = table.reset_global_filter(default_state.unwrap_or(false));
                }
                FixtureAction::ResetGrouping { default_state } => {
                    state.grouping = table.reset_grouping(default_state.unwrap_or(false));
                }
                FixtureAction::ResetColumnVisibility { default_state } => {
                    state.column_visibility =
                        table.reset_column_visibility(default_state.unwrap_or(false));
                }
                FixtureAction::ResetColumnOrder { default_state } => {
                    state.column_order = table.reset_column_order(default_state.unwrap_or(false));
                }
                FixtureAction::ResetRowSelection { default_state } => {
                    state.row_selection = table.reset_row_selection(default_state.unwrap_or(false));
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
            assert_eq!(
                state.column_filters, expected_state.column_filters,
                "snapshot {} next_state.columnFilters mismatch",
                snap.id
            );
            assert_eq!(
                state.global_filter, expected_state.global_filter,
                "snapshot {} next_state.globalFilter mismatch",
                snap.id
            );
            assert_eq!(
                state.grouping, expected_state.grouping,
                "snapshot {} next_state.grouping mismatch",
                snap.id
            );
            assert_eq!(
                state.column_visibility, expected_state.column_visibility,
                "snapshot {} next_state.columnVisibility mismatch",
                snap.id
            );
            assert_eq!(
                state.column_order, expected_state.column_order,
                "snapshot {} next_state.columnOrder mismatch",
                snap.id
            );
            assert_eq!(
                state.row_selection, expected_state.row_selection,
                "snapshot {} next_state.rowSelection mismatch",
                snap.id
            );
        } else {
            panic!("snapshot {} missing expect.next_state", snap.id);
        }
    }
}
