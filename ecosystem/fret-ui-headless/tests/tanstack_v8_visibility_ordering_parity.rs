use std::collections::HashMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, ColumnSizingRegion, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
    set_column_order_for,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureColumnMeta {
    id: String,
    size: f32,
    #[serde(default, rename = "minSize")]
    min_size: Option<f32>,
    #[serde(default, rename = "maxSize")]
    max_size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct LeafColumnsExpect {
    all: Vec<String>,
    visible: Vec<String>,
    left_visible: Vec<String>,
    center_visible: Vec<String>,
    right_visible: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ColumnNodeExpect {
    id: String,
    depth: usize,
    parent_id: Option<String>,
    child_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FlatColumnsExpect {
    all: Vec<String>,
    visible: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CoreModelExpect {
    #[serde(default)]
    column_tree: Vec<ColumnNodeExpect>,
    #[serde(default)]
    flat_columns: Option<FlatColumnsExpect>,
    leaf_columns: LeafColumnsExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnSizingExpect {
    total_size: f32,
    left_total_size: f32,
    center_total_size: f32,
    right_total_size: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnStartExpect {
    all: HashMap<String, f32>,
    left: HashMap<String, Option<f32>>,
    center: HashMap<String, Option<f32>>,
    right: HashMap<String, Option<f32>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnAfterExpect {
    all: HashMap<String, f32>,
    left: HashMap<String, Option<f32>>,
    center: HashMap<String, Option<f32>>,
    right: HashMap<String, Option<f32>>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    expanded: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    page_count: i32,
    row_count: usize,
    can_previous_page: bool,
    can_next_page: bool,
    page_options: Vec<usize>,
    selected: RowModelSnapshot,
    filtered_selected: RowModelSnapshot,
    grouped_selected: RowModelSnapshot,
    is_all_rows_selected: bool,
    is_some_rows_selected: bool,
    is_all_page_rows_selected: bool,
    is_some_page_rows_selected: bool,
    is_all_rows_expanded: bool,
    is_some_rows_expanded: bool,
    can_some_rows_expand: bool,
    is_all_columns_visible: bool,
    is_some_columns_visible: bool,
    #[serde(rename = "column_sizing")]
    sizing: ColumnSizingExpect,
    #[serde(rename = "column_start")]
    starts: ColumnStartExpect,
    #[serde(rename = "column_after")]
    after: ColumnAfterExpect,
    core_model: CoreModelExpect,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "toggleColumnVisibility")]
    ToggleColumnVisibility {
        column_id: String,
        #[serde(default)]
        value: Option<bool>,
    },
    #[serde(rename = "toggleAllColumnsVisible")]
    ToggleAllColumnsVisible {
        #[serde(default)]
        value: Option<bool>,
    },
    #[serde(rename = "setColumnOrder")]
    SetColumnOrder { order: Vec<String> },
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
    columns_meta: Vec<FixtureColumnMeta>,
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
fn tanstack_v8_visibility_ordering_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("visibility_ordering.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "visibility_ordering");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = fixture
        .columns_meta
        .iter()
        .map(|m| {
            let mut col = ColumnDef::<FixtureRow>::new(m.id.as_str())
                .size(m.size)
                .enable_hiding(m.id != "b")
                .sort_by(|_a: &FixtureRow, _b: &FixtureRow| std::cmp::Ordering::Equal);
            if let Some(min) = m.min_size {
                col = col.min_size(min);
            }
            if let Some(max) = m.max_size {
                col = col.max_size(max);
            }
            col
        })
        .collect();

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        let column_visibility_hook_noop = snap
            .options
            .get("__onColumnVisibilityChange")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == "noop");
        let column_order_hook_noop = snap
            .options
            .get("__onColumnOrderChange")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == "noop");

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::ToggleColumnVisibility { column_id, value } => {
                    if column_visibility_hook_noop {
                        continue;
                    }
                    state.column_visibility = table
                        .toggled_column_visibility(column_id.as_str(), *value)
                        .unwrap_or_else(|| panic!("unknown column in action: {column_id}"));
                }
                FixtureAction::ToggleAllColumnsVisible { value } => {
                    if column_visibility_hook_noop {
                        continue;
                    }
                    state.column_visibility = table.toggled_all_columns_visible(*value);
                }
                FixtureAction::SetColumnOrder { order } => {
                    if column_order_hook_noop {
                        continue;
                    }
                    set_column_order_for(
                        &mut state.column_order,
                        order.iter().map(|id| id.as_str()),
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
                state.column_visibility, expected_state.column_visibility,
                "snapshot {} next_state.columnVisibility mismatch",
                snap.id
            );
            assert_eq!(
                state.column_order, expected_state.column_order,
                "snapshot {} next_state.columnOrder mismatch",
                snap.id
            );
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
        let expanded = snapshot_row_model(table.expanded_row_model());
        let row_model = snapshot_row_model(table.row_model());

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

        assert_eq!(
            row_model.root, snap.expect.row_model.root,
            "snapshot {} row_model root mismatch",
            snap.id
        );
        assert_eq!(
            row_model.flat, snap.expect.row_model.flat,
            "snapshot {} row_model flat mismatch",
            snap.id
        );

        assert_eq!(
            row_model.root, snap.expect.paginated.root,
            "snapshot {} paginated root mismatch",
            snap.id
        );
        assert_eq!(
            row_model.flat, snap.expect.paginated.flat,
            "snapshot {} paginated flat mismatch",
            snap.id
        );

        assert_eq!(
            table.page_count(),
            snap.expect.page_count,
            "snapshot {} page_count mismatch",
            snap.id
        );
        assert_eq!(
            table.row_count(),
            snap.expect.row_count,
            "snapshot {} row_count mismatch",
            snap.id
        );
        assert_eq!(
            table.can_previous_page(),
            snap.expect.can_previous_page,
            "snapshot {} can_previous_page mismatch",
            snap.id
        );
        assert_eq!(
            table.can_next_page(),
            snap.expect.can_next_page,
            "snapshot {} can_next_page mismatch",
            snap.id
        );
        assert_eq!(
            table.page_options(),
            snap.expect.page_options,
            "snapshot {} page_options mismatch",
            snap.id
        );

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

        let grouped_selected = snapshot_row_model(table.grouped_selected_row_model());
        assert_eq!(
            grouped_selected.root, snap.expect.grouped_selected.root,
            "snapshot {} grouped_selected root mismatch",
            snap.id
        );
        assert_eq!(
            grouped_selected.flat, snap.expect.grouped_selected.flat,
            "snapshot {} grouped_selected flat mismatch",
            snap.id
        );

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
        assert_eq!(
            table.is_all_columns_visible(),
            snap.expect.is_all_columns_visible,
            "snapshot {} is_all_columns_visible mismatch",
            snap.id
        );
        assert_eq!(
            table.is_some_columns_visible(),
            snap.expect.is_some_columns_visible,
            "snapshot {} is_some_columns_visible mismatch",
            snap.id
        );

        assert_eq!(
            table.total_size(),
            snap.expect.sizing.total_size,
            "snapshot {} column_sizing.total_size mismatch",
            snap.id
        );
        assert_eq!(
            table.left_total_size(),
            snap.expect.sizing.left_total_size,
            "snapshot {} column_sizing.left_total_size mismatch",
            snap.id
        );
        assert_eq!(
            table.center_total_size(),
            snap.expect.sizing.center_total_size,
            "snapshot {} column_sizing.center_total_size mismatch",
            snap.id
        );
        assert_eq!(
            table.right_total_size(),
            snap.expect.sizing.right_total_size,
            "snapshot {} column_sizing.right_total_size mismatch",
            snap.id
        );

        for (col_id, expected) in &snap.expect.starts.all {
            let actual = table.column_start(col_id.as_str(), ColumnSizingRegion::All);
            assert_eq!(
                actual,
                Some(*expected),
                "snapshot {} column_start(all,{col_id}) mismatch",
                snap.id
            );
        }
        for (col_id, expected) in &snap.expect.after.all {
            let actual = table.column_after(col_id.as_str(), ColumnSizingRegion::All);
            assert_eq!(
                actual,
                Some(*expected),
                "snapshot {} column_after(all,{col_id}) mismatch",
                snap.id
            );
        }

        let starts = [
            (ColumnSizingRegion::Left, &snap.expect.starts.left, "left"),
            (
                ColumnSizingRegion::Center,
                &snap.expect.starts.center,
                "center",
            ),
            (
                ColumnSizingRegion::Right,
                &snap.expect.starts.right,
                "right",
            ),
        ];
        for (region, by_col, label) in starts {
            for (col_id, expected) in by_col {
                let actual = table.column_start(col_id.as_str(), region);
                assert_eq!(
                    actual, *expected,
                    "snapshot {} column_start({label},{col_id}) mismatch",
                    snap.id
                );
            }
        }

        let after = [
            (ColumnSizingRegion::Left, &snap.expect.after.left, "left"),
            (
                ColumnSizingRegion::Center,
                &snap.expect.after.center,
                "center",
            ),
            (ColumnSizingRegion::Right, &snap.expect.after.right, "right"),
        ];
        for (region, by_col, label) in after {
            for (col_id, expected) in by_col {
                let actual = table.column_after(col_id.as_str(), region);
                assert_eq!(
                    actual, *expected,
                    "snapshot {} column_after({label},{col_id}) mismatch",
                    snap.id
                );
            }
        }

        let model = table.core_model_snapshot();
        let actual = LeafColumnsExpect {
            all: model
                .leaf_columns
                .all
                .iter()
                .map(|s| s.to_string())
                .collect(),
            visible: model
                .leaf_columns
                .visible
                .iter()
                .map(|s| s.to_string())
                .collect(),
            left_visible: model
                .leaf_columns
                .left_visible
                .iter()
                .map(|s| s.to_string())
                .collect(),
            center_visible: model
                .leaf_columns
                .center_visible
                .iter()
                .map(|s| s.to_string())
                .collect(),
            right_visible: model
                .leaf_columns
                .right_visible
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };

        assert_eq!(
            actual.all, snap.expect.core_model.leaf_columns.all,
            "snapshot {} core_model.leaf_columns.all mismatch",
            snap.id
        );
        assert_eq!(
            actual.visible, snap.expect.core_model.leaf_columns.visible,
            "snapshot {} core_model.leaf_columns.visible mismatch",
            snap.id
        );
        assert_eq!(
            actual.left_visible, snap.expect.core_model.leaf_columns.left_visible,
            "snapshot {} core_model.leaf_columns.left_visible mismatch",
            snap.id
        );
        assert_eq!(
            actual.center_visible, snap.expect.core_model.leaf_columns.center_visible,
            "snapshot {} core_model.leaf_columns.center_visible mismatch",
            snap.id
        );
        assert_eq!(
            actual.right_visible, snap.expect.core_model.leaf_columns.right_visible,
            "snapshot {} core_model.leaf_columns.right_visible mismatch",
            snap.id
        );

        if !snap.expect.core_model.column_tree.is_empty() {
            let column_tree: Vec<ColumnNodeExpect> = table
                .column_tree_snapshot()
                .into_iter()
                .map(|n| ColumnNodeExpect {
                    id: n.id.as_ref().to_string(),
                    depth: n.depth,
                    parent_id: n.parent_id.as_ref().map(|s| s.as_ref().to_string()),
                    child_ids: n
                        .child_ids
                        .into_iter()
                        .map(|s| s.as_ref().to_string())
                        .collect(),
                })
                .collect();

            assert_eq!(
                column_tree, snap.expect.core_model.column_tree,
                "snapshot {} core_model.column_tree mismatch",
                snap.id
            );

            for expected in &snap.expect.core_model.column_tree {
                let got = table
                    .column_node_snapshot(expected.id.as_str())
                    .unwrap_or_else(|| panic!("unknown column id: {}", expected.id));
                let got = ColumnNodeExpect {
                    id: got.id.as_ref().to_string(),
                    depth: got.depth,
                    parent_id: got.parent_id.as_ref().map(|s| s.as_ref().to_string()),
                    child_ids: got
                        .child_ids
                        .into_iter()
                        .map(|s| s.as_ref().to_string())
                        .collect(),
                };
                assert_eq!(
                    got,
                    expected.clone(),
                    "snapshot {} column_node_snapshot({}) mismatch",
                    snap.id,
                    expected.id
                );
            }
        }

        if let Some(expected) = snap.expect.core_model.flat_columns.as_ref() {
            let all_flat: Vec<String> = table
                .all_flat_columns()
                .into_iter()
                .map(|c| c.id.to_string())
                .collect();
            let visible_flat: Vec<String> = table
                .visible_flat_columns()
                .into_iter()
                .map(|c| c.id.to_string())
                .collect();

            assert_eq!(
                all_flat, expected.all,
                "snapshot {} core_model.flat_columns.all mismatch",
                snap.id
            );
            assert_eq!(
                visible_flat, expected.visible,
                "snapshot {} core_model.flat_columns.visible mismatch",
                snap.id
            );
        }
    }
}
