use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, ExpandingState, RowId, RowKey, RowPinPosition, Table, TanStackTableOptions,
    TanStackTableState,
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
    expanded: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    selected: RowModelSnapshot,
    filtered_selected: RowModelSnapshot,
    is_all_rows_selected: bool,
    is_some_rows_selected: bool,
    is_all_page_rows_selected: bool,
    is_some_page_rows_selected: bool,
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
    #[serde(rename = "toggleRowExpanded")]
    ToggleRowExpanded {
        row_id: String,
        #[serde(default)]
        value: Option<bool>,
    },
    #[serde(rename = "pinRow")]
    PinRow {
        row_id: String,
        position: Option<String>,
        #[serde(default)]
        include_leaf_rows: bool,
        #[serde(default)]
        include_parent_rows: bool,
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
        .filter_map(|&i| model.row(i).map(|r| r.id.as_str().to_string()))
        .collect();
    let flat = model
        .flat_rows()
        .iter()
        .filter_map(|&i| model.row(i).map(|r| r.id.as_str().to_string()))
        .collect();
    RowModelSnapshot { root, flat }
}

#[test]
fn tanstack_v8_row_id_state_ops_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("row_id_state_ops.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "row_id_state_ops");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name").facet_str_by(|row: &FixtureRow| row.name.as_str()),
        ColumnDef::<FixtureRow>::new("status").facet_str_by(|row: &FixtureRow| row.status.as_str()),
        ColumnDef::<FixtureRow>::new("cpu").value_u64_by(|row: &FixtureRow| row.cpu),
        ColumnDef::<FixtureRow>::new("mem_mb").value_u64_by(|row: &FixtureRow| row.mem_mb),
    ];

    let column_by_id: HashMap<&str, &ColumnDef<FixtureRow>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state
            .to_table_state_with_row_model(
                Table::builder(&data)
                    .columns(columns.clone())
                    .get_row_key(|row, _idx, _parent| RowKey(row.id))
                    .get_row_id(|row, _idx, _parent| {
                        if snap
                            .options
                            .get("__getRowId")
                            .and_then(|v| v.as_str())
                            .is_some_and(|v| v == "prefixed")
                        {
                            RowId::new(format!("row:{}", row.id))
                        } else {
                            RowId::new(row.id.to_string())
                        }
                    })
                    .state(Default::default())
                    .build()
                    .core_row_model(),
            )
            .expect("state conversion with row model");

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| {
                    if snap
                        .options
                        .get("__getRowId")
                        .and_then(|v| v.as_str())
                        .is_some_and(|v| v == "prefixed")
                    {
                        RowId::new(format!("row:{}", row.id))
                    } else {
                        RowId::new(row.id.to_string())
                    }
                })
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::ToggleRowSelected {
                    row_id,
                    value,
                    select_children,
                } => {
                    state.row_selection = table
                        .toggled_row_selected_by_id(row_id, true, *value, *select_children)
                        .unwrap_or_else(|| panic!("unknown row id for selection: {row_id}"));
                }
                FixtureAction::ToggleRowExpanded { row_id, value } => {
                    state.expanding = table
                        .toggled_row_expanded_by_id(row_id, true, *value)
                        .unwrap_or_else(|| panic!("unknown row id for expanding: {row_id}"));
                }
                FixtureAction::PinRow {
                    row_id,
                    position,
                    include_leaf_rows,
                    include_parent_rows,
                } => {
                    let position = match position.as_deref() {
                        Some("top") => Some(RowPinPosition::Top),
                        Some("bottom") => Some(RowPinPosition::Bottom),
                        None => None,
                        Some(other) => panic!("unsupported pin position: {other}"),
                    };
                    let updater = table
                        .row_pinning_updater_by_id(
                            row_id,
                            true,
                            position,
                            *include_leaf_rows,
                            *include_parent_rows,
                        )
                        .unwrap_or_else(|| panic!("unknown row id for pinning: {row_id}"));
                    state.row_pinning = updater.apply(&state.row_pinning);
                }
            }

            state.column_order = state
                .column_order
                .iter()
                .filter(|id| column_by_id.contains_key(id.as_ref()))
                .cloned()
                .collect();
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| {
                if snap
                    .options
                    .get("__getRowId")
                    .and_then(|v| v.as_str())
                    .is_some_and(|v| v == "prefixed")
                {
                    RowId::new(format!("row:{}", row.id))
                } else {
                    RowId::new(row.id.to_string())
                }
            })
            .state(state.clone())
            .options(options)
            .build();

        if let Some(expected_next) = snap.expect.next_state.as_ref() {
            let expected_row_selection = expected_next
                .get("rowSelection")
                .and_then(|v| v.as_object())
                .map(|map| {
                    map.iter()
                        .filter(|(_id, enabled)| enabled.as_bool().unwrap_or(false))
                        .map(|(id, _enabled)| {
                            table.row_key_for_id(id, true).unwrap_or_else(|| {
                                panic!("unresolved next_state rowSelection id: {id}")
                            })
                        })
                        .collect::<HashSet<_>>()
                })
                .unwrap_or_default();

            let expected_row_pinning_top = expected_next
                .get("rowPinning")
                .and_then(|v| v.as_object())
                .and_then(|obj| obj.get("top"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|id| {
                            table.row_key_for_id(id, true).unwrap_or_else(|| {
                                panic!("unresolved next_state rowPinning.top id: {id}")
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let expected_row_pinning_bottom = expected_next
                .get("rowPinning")
                .and_then(|v| v.as_object())
                .and_then(|obj| obj.get("bottom"))
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .map(|id| {
                            table.row_key_for_id(id, true).unwrap_or_else(|| {
                                panic!("unresolved next_state rowPinning.bottom id: {id}")
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let expected_expanding = match expected_next.get("expanded") {
                Some(serde_json::Value::Bool(true)) => ExpandingState::All,
                Some(serde_json::Value::Bool(false)) | None | Some(serde_json::Value::Null) => {
                    ExpandingState::default()
                }
                Some(serde_json::Value::Object(map)) => ExpandingState::from_iter(
                    map.iter()
                        .filter(|(_id, enabled)| enabled.as_bool().unwrap_or(false))
                        .map(|(id, _enabled)| {
                            table.row_key_for_id(id, true).unwrap_or_else(|| {
                                panic!("unresolved next_state expanded id: {id}")
                            })
                        }),
                ),
                Some(other) => panic!("unsupported next_state.expanded shape: {other:?}"),
            };

            assert_eq!(
                state.row_selection, expected_row_selection,
                "snapshot {} next_state.rowSelection mismatch",
                snap.id
            );
            assert_eq!(
                state.row_pinning.top, expected_row_pinning_top,
                "snapshot {} next_state.rowPinning.top mismatch",
                snap.id
            );
            assert_eq!(
                state.row_pinning.bottom, expected_row_pinning_bottom,
                "snapshot {} next_state.rowPinning.bottom mismatch",
                snap.id
            );
            assert_eq!(
                state.expanding, expected_expanding,
                "snapshot {} next_state.expanded mismatch",
                snap.id
            );
        }

        let core = snapshot_row_model(table.core_row_model());
        let filtered = snapshot_row_model(table.filtered_row_model());
        let sorted = snapshot_row_model(table.sorted_row_model());
        let expanded = snapshot_row_model(table.expanded_row_model());
        let paginated = snapshot_row_model(table.row_model());
        let grouping_active = !table.state().grouping.is_empty();

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
        if !grouping_active {
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

        let selected = snapshot_row_model(table.selected_row_model());
        assert_eq!(
            selected.root, snap.expect.selected.root,
            "snapshot {} selected root mismatch",
            snap.id
        );
        assert_eq!(
            selected.flat, snap.expect.selected.flat,
            "snapshot {} selected flat mismatch",
            snap.id
        );

        let filtered_selected = snapshot_row_model(table.filtered_selected_row_model());
        assert_eq!(
            filtered_selected.root, snap.expect.filtered_selected.root,
            "snapshot {} filtered_selected root mismatch",
            snap.id
        );
        assert_eq!(
            filtered_selected.flat, snap.expect.filtered_selected.flat,
            "snapshot {} filtered_selected flat mismatch",
            snap.id
        );

        if !grouping_active {
            assert_eq!(
                table.is_all_rows_selected(),
                snap.expect.is_all_rows_selected,
                "snapshot {} is_all_rows_selected mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_rows_selected(),
                snap.expect.is_some_rows_selected,
                "snapshot {} is_some_rows_selected mismatch",
                snap.id
            );
            assert_eq!(
                table.is_all_page_rows_selected(),
                snap.expect.is_all_page_rows_selected,
                "snapshot {} is_all_page_rows_selected mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_page_rows_selected(),
                snap.expect.is_some_page_rows_selected,
                "snapshot {} is_some_page_rows_selected mismatch",
                snap.id
            );
        }

        if snap.id == "row_id_state_ops_group_expanding" {
            assert!(
                matches!(state.expanding, ExpandingState::Keys(ref keys) if !keys.is_empty()),
                "snapshot {} expected expanded keys to be non-empty",
                snap.id
            );
        }
    }
}
