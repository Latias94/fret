use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    Aggregation, ColumnDef, GroupedRowKind, GroupedRowModel, RowId, RowKey, RowPinPosition, Table,
    TableState, TanStackTableOptions, TanStackTableState, grouped_row_model_from_leaf,
    sort_grouped_row_indices_in_place,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: u64,
    team: u64,
    score: u64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct PathEntry {
    column_id: String,
    value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GroupedRowNodeExpect {
    kind: String,
    depth: usize,
    path: Vec<PathEntry>,
    #[serde(default)]
    grouping_column_id: Option<String>,
    #[serde(default)]
    grouping_value: Option<serde_json::Value>,
    #[serde(default)]
    leaf_row_count: Option<usize>,
    #[serde(default)]
    first_leaf_row_id: Option<String>,
    #[serde(default)]
    row_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GroupedRowModelExpect {
    root: Vec<GroupedRowNodeExpect>,
    flat: Vec<GroupedRowNodeExpect>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GroupedAggregationU64Expect {
    path: Vec<PathEntry>,
    values: BTreeMap<String, Option<u64>>,
}

#[derive(Debug, Clone, Deserialize)]
struct RowPinningExpect {
    top: Vec<String>,
    center: Vec<String>,
    bottom: Vec<String>,
    #[serde(default)]
    can_pin: BTreeMap<String, bool>,
    #[serde(default)]
    pin_position: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pinned_index: BTreeMap<String, i32>,
    is_some_rows_pinned: bool,
    is_some_top_rows_pinned: bool,
    is_some_bottom_rows_pinned: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    #[serde(default)]
    grouped_row_model: Option<GroupedRowModelExpect>,
    #[serde(default)]
    grouped_aggregations_u64: Option<Vec<GroupedAggregationU64Expect>>,
    #[serde(default)]
    sorted_grouped_row_model: Option<GroupedRowModelExpect>,
    #[serde(default)]
    row_pinning: Option<RowPinningExpect>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "toggleGrouping")]
    ToggleGrouping {
        column_id: String,
        #[serde(default)]
        value: Option<bool>,
    },
    #[serde(rename = "toggleGroupingHandler")]
    ToggleGroupingHandler { column_id: String },
    #[serde(rename = "setGrouping")]
    SetGrouping { grouping: Vec<String> },
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

fn grouped_node_path(model: &GroupedRowModel, mut node: usize) -> Vec<(Arc<str>, String)> {
    let mut out_rev: Vec<(Arc<str>, String)> = Vec::new();
    loop {
        let Some(row) = model.row(node) else {
            break;
        };
        match &row.kind {
            GroupedRowKind::Group {
                grouping_column,
                grouping_value,
                ..
            } => {
                out_rev.push((grouping_column.clone(), grouping_value.to_string()));
            }
            GroupedRowKind::Leaf { .. } => {}
        }
        let Some(parent) = row.parent else {
            break;
        };
        node = parent;
    }
    out_rev.reverse();
    out_rev
}

fn snapshot_node(model: &GroupedRowModel, index: usize) -> GroupedRowNodeExpect {
    let row = model.row(index).expect("grouped row exists");
    let path = grouped_node_path(model, index)
        .into_iter()
        .map(|(column_id, value)| PathEntry {
            column_id: column_id.to_string(),
            value: serde_json::Value::String(value),
        })
        .collect::<Vec<_>>();

    match &row.kind {
        GroupedRowKind::Group {
            grouping_column,
            grouping_value,
            first_leaf_row_key,
            leaf_row_count,
        } => GroupedRowNodeExpect {
            kind: "group".into(),
            depth: row.depth,
            path,
            grouping_column_id: Some(grouping_column.to_string()),
            grouping_value: Some(serde_json::Value::String(grouping_value.to_string())),
            leaf_row_count: Some(*leaf_row_count),
            first_leaf_row_id: Some(first_leaf_row_key.0.to_string()),
            row_id: None,
        },
        GroupedRowKind::Leaf { row_key } => GroupedRowNodeExpect {
            kind: "leaf".into(),
            depth: row.depth,
            path,
            grouping_column_id: None,
            grouping_value: None,
            leaf_row_count: None,
            first_leaf_row_id: None,
            row_id: Some(row_key.0.to_string()),
        },
    }
}

fn snapshot_root_preorder(model: &GroupedRowModel) -> Vec<GroupedRowNodeExpect> {
    fn walk(model: &GroupedRowModel, index: usize, out: &mut Vec<GroupedRowNodeExpect>) {
        out.push(snapshot_node(model, index));
        let Some(row) = model.row(index) else {
            return;
        };
        for &child in &row.sub_rows {
            walk(model, child, out);
        }
    }

    let mut out = Vec::new();
    for &root in model.root_rows() {
        walk(model, root, &mut out);
    }
    out
}

fn group_nodes_with_path(model: &GroupedRowModel) -> Vec<(RowKey, Vec<PathEntry>)> {
    fn walk(
        model: &GroupedRowModel,
        index: usize,
        path: &[PathEntry],
        out: &mut Vec<(RowKey, Vec<PathEntry>)>,
    ) {
        let Some(row) = model.row(index) else {
            return;
        };

        let (next_path, is_group) = match &row.kind {
            GroupedRowKind::Group {
                grouping_column,
                grouping_value,
                ..
            } => {
                let mut next = path.to_vec();
                next.push(PathEntry {
                    column_id: grouping_column.to_string(),
                    value: serde_json::Value::String(grouping_value.to_string()),
                });
                (next, true)
            }
            GroupedRowKind::Leaf { .. } => (path.to_vec(), false),
        };

        if is_group {
            out.push((row.key, next_path.clone()));
        }

        for &child in &row.sub_rows {
            walk(model, child, next_path.as_slice(), out);
        }
    }

    let mut out = Vec::new();
    for &root in model.root_rows() {
        walk(model, root, &[], &mut out);
    }
    out
}

fn path_key(path: &[PathEntry]) -> String {
    path.iter()
        .map(|e| {
            let v = e
                .value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| e.value.to_string());
            format!("{}={}", e.column_id, v)
        })
        .collect::<Vec<_>>()
        .join("|")
}

fn snapshot_sorted_grouped_row_model<TData>(
    model: &GroupedRowModel,
    sorting: &[fret_ui_headless::table::SortSpec],
    columns: &[ColumnDef<TData>],
    data: &[TData],
    row_index_by_key: &std::collections::HashMap<RowKey, usize>,
    group_aggs_u64: &std::collections::HashMap<RowKey, Arc<[(Arc<str>, u64)]>>,
    group_aggs_any: &std::collections::HashMap<
        RowKey,
        Arc<[(Arc<str>, fret_ui_headless::table::TanStackValue)]>,
    >,
) -> (Vec<GroupedRowNodeExpect>, Vec<GroupedRowNodeExpect>) {
    let mut roots: Vec<usize> = model.root_rows().to_vec();

    sort_grouped_row_indices_in_place(
        model,
        roots.as_mut_slice(),
        sorting,
        columns,
        data,
        row_index_by_key,
        group_aggs_u64,
        group_aggs_any,
    );

    fn walk_root<TData>(
        model: &GroupedRowModel,
        indices: &[usize],
        sorting: &[fret_ui_headless::table::SortSpec],
        columns: &[ColumnDef<TData>],
        data: &[TData],
        row_index_by_key: &std::collections::HashMap<RowKey, usize>,
        group_aggs_u64: &std::collections::HashMap<RowKey, Arc<[(Arc<str>, u64)]>>,
        group_aggs_any: &std::collections::HashMap<
            RowKey,
            Arc<[(Arc<str>, fret_ui_headless::table::TanStackValue)]>,
        >,
        out: &mut Vec<GroupedRowNodeExpect>,
    ) {
        for &index in indices {
            out.push(snapshot_node(model, index));
            let Some(row) = model.row(index) else {
                continue;
            };
            if row.sub_rows.is_empty() {
                continue;
            }
            let mut children = row.sub_rows.clone();
            sort_grouped_row_indices_in_place(
                model,
                children.as_mut_slice(),
                sorting,
                columns,
                data,
                row_index_by_key,
                group_aggs_u64,
                group_aggs_any,
            );
            walk_root(
                model,
                children.as_slice(),
                sorting,
                columns,
                data,
                row_index_by_key,
                group_aggs_u64,
                group_aggs_any,
                out,
            );
        }
    }

    let mut root: Vec<GroupedRowNodeExpect> = Vec::new();
    walk_root(
        model,
        roots.as_slice(),
        sorting,
        columns,
        data,
        row_index_by_key,
        group_aggs_u64,
        group_aggs_any,
        &mut root,
    );

    let flat = if sorting.is_empty() {
        // TanStack `getSortedRowModel` returns the pre-sorted model when no sorting is active,
        // which preserves the grouped row model's `flatRows` behavior (including duplicates).
        model
            .flat_rows()
            .iter()
            .copied()
            .map(|i| snapshot_node(model, i))
            .collect::<Vec<_>>()
    } else {
        // When sorting is active, TanStack rebuilds `flatRows` from the sorted `rows` tree.
        root.clone()
    };

    (root, flat)
}

#[test]
fn tanstack_v8_grouping_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("grouping.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "grouping");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("role").facet_key_by(|row: &FixtureRow| row.role),
        ColumnDef::<FixtureRow>::new("team").facet_key_by(|row: &FixtureRow| row.team),
        ColumnDef::<FixtureRow>::new("score")
            .value_u64_by(|row: &FixtureRow| row.score)
            .aggregate(Aggregation::SumU64),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        // TanStack: table-level reset APIs target `options.initialState`, not `options.state`.
        // Auto reset behaviors (e.g. `autoResetExpanded`) also reset to `initialState`.
        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        let grouped_override_pre_grouped = snap
            .options
            .get("__getGroupedRowModel")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == "pre_grouped");

        let mut auto_reset_expanded_registered = false;
        let mut auto_reset_page_index_registered = false;

        for action in &snap.actions {
            let prev_grouping = state.grouping.clone();

            let mut builder = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options);

            if grouped_override_pre_grouped {
                builder = builder.get_grouped_row_model(|pre, _cols, _grouping| {
                    grouped_row_model_from_leaf(pre)
                });
            }

            let table = builder.build();

            match action {
                FixtureAction::ToggleGrouping { column_id, value } => {
                    let updater = table
                        .grouping_updater(column_id, *value)
                        .unwrap_or_else(|| panic!("unknown grouping column: {column_id}"));
                    state.grouping = updater.apply(&state.grouping);
                }
                FixtureAction::ToggleGroupingHandler { column_id } => {
                    let updater = table
                        .grouping_handler_updater(column_id)
                        .unwrap_or_else(|| panic!("unknown grouping column: {column_id}"));
                    state.grouping = updater.apply(&state.grouping);
                }
                FixtureAction::SetGrouping { grouping } => {
                    state.grouping = grouping
                        .iter()
                        .map(|v| Arc::<str>::from(v.as_str()))
                        .collect();
                }
            }

            let grouping_changed = state.grouping != prev_grouping;
            if grouping_changed {
                // TanStack Table v8: the grouped row model memo debug callback queues
                // `_autoResetExpanded()`. The first invocation only registers; subsequent invocations
                // can trigger a reset depending on option gates.
                if !auto_reset_expanded_registered {
                    auto_reset_expanded_registered = true;
                } else if table.should_auto_reset_expanded() {
                    state.expanding = table.reset_expanded(false);
                }

                // TanStack Table v8: the grouped row model memo debug callback also queues
                // `_autoResetPageIndex()` (RowPagination feature). Mirrors the same "first call
                // registers, subsequent calls may reset" behavior.
                if !auto_reset_page_index_registered {
                    auto_reset_page_index_registered = true;
                } else if table.should_auto_reset_page_index() {
                    state.pagination = table.reset_page_index(false);
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
                state.grouping, expected_state.grouping,
                "snapshot {} next_state.grouping mismatch",
                snap.id
            );
            assert_eq!(
                state.expanding, expected_state.expanding,
                "snapshot {} next_state.expanded mismatch",
                snap.id
            );
            assert_eq!(
                state.pagination, expected_state.pagination,
                "snapshot {} next_state.pagination mismatch",
                snap.id
            );
        }

        let mut builder = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options);

        if grouped_override_pre_grouped {
            builder = builder
                .get_grouped_row_model(|pre, _cols, _grouping| grouped_row_model_from_leaf(pre));
        }

        let table = builder.build();

        if let Some(expected) = snap.expect.row_pinning.as_ref() {
            let top: Vec<String> = table
                .top_row_ids()
                .into_iter()
                .map(|id| id.as_str().to_string())
                .collect();
            let center: Vec<String> = table
                .center_row_ids()
                .into_iter()
                .map(|id| id.as_str().to_string())
                .collect();
            let bottom: Vec<String> = table
                .bottom_row_ids()
                .into_iter()
                .map(|id| id.as_str().to_string())
                .collect();

            assert_eq!(
                top, expected.top,
                "snapshot {} row_pinning.top mismatch",
                snap.id
            );
            assert_eq!(
                center, expected.center,
                "snapshot {} row_pinning.center mismatch",
                snap.id
            );
            assert_eq!(
                bottom, expected.bottom,
                "snapshot {} row_pinning.bottom mismatch",
                snap.id
            );

            for (row_id, expected_can_pin) in &expected.can_pin {
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                let can_pin = table
                    .row_can_pin(row_key)
                    .unwrap_or_else(|| panic!("unknown row: {row_id}"));
                assert_eq!(
                    can_pin, *expected_can_pin,
                    "snapshot {} can_pin[{}] mismatch",
                    snap.id, row_id
                );
            }

            for (row_id, expected_pos) in &expected.pin_position {
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                let pos = table.row_is_pinned(row_key).map(|p| match p {
                    RowPinPosition::Top => "top",
                    RowPinPosition::Bottom => "bottom",
                });
                assert_eq!(
                    pos.as_deref(),
                    expected_pos.as_deref(),
                    "snapshot {} pin_position[{}] mismatch",
                    snap.id,
                    row_id
                );
            }

            for (row_id, expected_index) in &expected.pinned_index {
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                let index = table
                    .row_pinned_index(row_key)
                    .unwrap_or_else(|| panic!("unknown row: {row_id}"));
                assert_eq!(
                    index, *expected_index,
                    "snapshot {} pinned_index[{}] mismatch",
                    snap.id, row_id
                );
            }

            assert_eq!(
                table.is_some_rows_pinned(None),
                expected.is_some_rows_pinned,
                "snapshot {} is_some_rows_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_rows_pinned(Some(RowPinPosition::Top)),
                expected.is_some_top_rows_pinned,
                "snapshot {} is_some_top_rows_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_rows_pinned(Some(RowPinPosition::Bottom)),
                expected.is_some_bottom_rows_pinned,
                "snapshot {} is_some_bottom_rows_pinned mismatch",
                snap.id
            );
        }

        let actual_model = table.grouped_row_model();
        let actual_aggs = table.grouped_u64_aggregations();

        let mut row_index_by_key: std::collections::HashMap<RowKey, usize> = Default::default();
        for (i, row) in data.iter().enumerate() {
            row_index_by_key.insert(RowKey(row.id), i);
        }

        if let Some(expected) = snap.expect.grouped_row_model.as_ref() {
            let actual_root = snapshot_root_preorder(actual_model);
            let actual_flat = actual_model
                .flat_rows()
                .iter()
                .copied()
                .map(|i| snapshot_node(actual_model, i))
                .collect::<Vec<_>>();

            assert_eq!(
                actual_root.len(),
                expected.root.len(),
                "snapshot {} grouped_row_model.root length mismatch",
                snap.id
            );
            assert_eq!(
                actual_flat.len(),
                expected.flat.len(),
                "snapshot {} grouped_row_model.flat length mismatch",
                snap.id
            );

            for (i, (a, e)) in actual_root.iter().zip(expected.root.iter()).enumerate() {
                assert_eq!(
                    a.kind, e.kind,
                    "snapshot {} root[{}].kind mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.depth, e.depth,
                    "snapshot {} root[{}].depth mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.path, e.path,
                    "snapshot {} root[{}].path mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.grouping_column_id, e.grouping_column_id,
                    "snapshot {} root[{}].grouping_column_id mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.grouping_value, e.grouping_value,
                    "snapshot {} root[{}].grouping_value mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.leaf_row_count, e.leaf_row_count,
                    "snapshot {} root[{}].leaf_row_count mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.first_leaf_row_id, e.first_leaf_row_id,
                    "snapshot {} root[{}].first_leaf_row_id mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.row_id, e.row_id,
                    "snapshot {} root[{}].row_id mismatch",
                    snap.id, i
                );
            }

            for (i, (a, e)) in actual_flat.iter().zip(expected.flat.iter()).enumerate() {
                assert_eq!(
                    a.kind, e.kind,
                    "snapshot {} flat[{}].kind mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.depth, e.depth,
                    "snapshot {} flat[{}].depth mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.path, e.path,
                    "snapshot {} flat[{}].path mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.grouping_column_id, e.grouping_column_id,
                    "snapshot {} flat[{}].grouping_column_id mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.grouping_value, e.grouping_value,
                    "snapshot {} flat[{}].grouping_value mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.leaf_row_count, e.leaf_row_count,
                    "snapshot {} flat[{}].leaf_row_count mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.first_leaf_row_id, e.first_leaf_row_id,
                    "snapshot {} flat[{}].first_leaf_row_id mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.row_id, e.row_id,
                    "snapshot {} flat[{}].row_id mismatch",
                    snap.id, i
                );
            }

            let expected_aggs = snap
                .expect
                .grouped_aggregations_u64
                .clone()
                .unwrap_or_default();
            let mut expected_aggs = expected_aggs;
            expected_aggs.sort_by_key(|e| path_key(&e.path));

            let mut actual_entries: Vec<GroupedAggregationU64Expect> = Vec::new();
            for (key, path) in group_nodes_with_path(actual_model) {
                let mut values: BTreeMap<String, Option<u64>> = BTreeMap::new();
                if let Some(entries) = actual_aggs.get(&key) {
                    for (col_id, v) in entries.iter() {
                        values.insert(col_id.to_string(), Some(*v));
                    }
                }
                actual_entries.push(GroupedAggregationU64Expect { path, values });
            }
            actual_entries.sort_by_key(|e| path_key(&e.path));

            assert_eq!(
                actual_entries.len(),
                expected_aggs.len(),
                "snapshot {} grouped_aggregations_u64 length mismatch",
                snap.id
            );

            for (i, (a, e)) in actual_entries.iter().zip(expected_aggs.iter()).enumerate() {
                assert_eq!(
                    a.path, e.path,
                    "snapshot {} grouped_aggregations_u64[{}].path mismatch",
                    snap.id, i
                );
                assert_eq!(
                    a.values, e.values,
                    "snapshot {} grouped_aggregations_u64[{}].values mismatch",
                    snap.id, i
                );
            }

            if let Some(expected_sorted) = snap.expect.sorted_grouped_row_model.as_ref() {
                let (actual_sorted_root, actual_sorted_flat) = snapshot_sorted_grouped_row_model(
                    actual_model,
                    &table.state().sorting,
                    columns.as_slice(),
                    data.as_slice(),
                    &row_index_by_key,
                    actual_aggs,
                    table.grouped_aggregations_any(),
                );

                assert_eq!(
                    actual_sorted_root.len(),
                    expected_sorted.root.len(),
                    "snapshot {} sorted_grouped_row_model.root length mismatch",
                    snap.id
                );
                assert_eq!(
                    actual_sorted_flat.len(),
                    expected_sorted.flat.len(),
                    "snapshot {} sorted_grouped_row_model.flat length mismatch",
                    snap.id
                );

                for (i, (a, e)) in actual_sorted_root
                    .iter()
                    .zip(expected_sorted.root.iter())
                    .enumerate()
                {
                    assert_eq!(
                        a, e,
                        "snapshot {} sorted_grouped_row_model.root[{}] mismatch",
                        snap.id, i
                    );
                }

                for (i, (a, e)) in actual_sorted_flat
                    .iter()
                    .zip(expected_sorted.flat.iter())
                    .enumerate()
                {
                    assert_eq!(
                        a, e,
                        "snapshot {} sorted_grouped_row_model.flat[{}] mismatch",
                        snap.id, i
                    );
                }
            }
        } else {
            panic!("snapshot {} missing grouped_row_model expectation", snap.id);
        }
    }
}
