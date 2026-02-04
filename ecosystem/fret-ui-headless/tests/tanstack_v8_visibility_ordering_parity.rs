use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowKey, Table, TanStackTableOptions, TanStackTableState, set_column_order_for,
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

#[derive(Debug, Clone, Deserialize)]
struct CoreModelExpect {
    leaf_columns: LeafColumnsExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
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

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::ToggleColumnVisibility { column_id, value } => {
                    state.column_visibility = table
                        .toggled_column_visibility(column_id.as_str(), *value)
                        .unwrap_or_else(|| panic!("unknown column in action: {column_id}"));
                }
                FixtureAction::ToggleAllColumnsVisible { value } => {
                    state.column_visibility = table.toggled_all_columns_visible(*value);
                }
                FixtureAction::SetColumnOrder { order } => {
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
            .state(state)
            .options(options)
            .build();

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
    }
}
