use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    cpu: u64,
    #[allow(dead_code)]
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct HeaderSnapshot {
    id: String,
    column_id: String,
    depth: usize,
    index: usize,
    is_placeholder: bool,
    placeholder_id: Option<String>,
    col_span: usize,
    row_span: usize,
    sub_header_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct HeaderGroupSnapshot {
    id: String,
    depth: usize,
    headers: Vec<HeaderSnapshot>,
}

fn header_to_jsonish(h: fret_ui_headless::table::HeaderSnapshot) -> HeaderSnapshot {
    HeaderSnapshot {
        id: h.id.as_ref().to_string(),
        column_id: h.column_id.as_ref().to_string(),
        depth: h.depth,
        index: h.index,
        is_placeholder: h.is_placeholder,
        placeholder_id: h.placeholder_id.as_ref().map(|s| s.as_ref().to_string()),
        col_span: h.col_span,
        row_span: h.row_span,
        sub_header_ids: h
            .sub_header_ids
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect(),
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CellSnapshot {
    id: String,
    column_id: String,
    #[serde(default)]
    is_grouped: bool,
    #[serde(default)]
    is_placeholder: bool,
    #[serde(default)]
    is_aggregated: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct RowCellsSnapshot {
    all: Vec<CellSnapshot>,
    visible: Vec<CellSnapshot>,
    left: Vec<CellSnapshot>,
    center: Vec<CellSnapshot>,
    right: Vec<CellSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct HeadersCellsExpect {
    header_groups: Vec<HeaderGroupSnapshot>,
    footer_groups: Vec<HeaderGroupSnapshot>,
    left_header_groups: Vec<HeaderGroupSnapshot>,
    left_footer_groups: Vec<HeaderGroupSnapshot>,
    center_header_groups: Vec<HeaderGroupSnapshot>,
    center_footer_groups: Vec<HeaderGroupSnapshot>,
    right_header_groups: Vec<HeaderGroupSnapshot>,
    right_footer_groups: Vec<HeaderGroupSnapshot>,
    flat_headers: Vec<HeaderSnapshot>,
    left_flat_headers: Vec<HeaderSnapshot>,
    center_flat_headers: Vec<HeaderSnapshot>,
    right_flat_headers: Vec<HeaderSnapshot>,
    leaf_headers: Vec<HeaderSnapshot>,
    left_leaf_headers: Vec<HeaderSnapshot>,
    center_leaf_headers: Vec<HeaderSnapshot>,
    right_leaf_headers: Vec<HeaderSnapshot>,
    cells: BTreeMap<String, RowCellsSnapshot>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ColumnNodeSnapshot {
    id: String,
    depth: usize,
    parent_id: Option<String>,
    child_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LeafColumnsSnapshot {
    all: Vec<String>,
    visible: Vec<String>,
    left_visible: Vec<String>,
    center_visible: Vec<String>,
    right_visible: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ColumnPinPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct ColumnCapabilitySnapshot {
    can_hide: bool,
    can_pin: bool,
    pin_position: Option<ColumnPinPosition>,
    pinned_index: i32,
    can_resize: bool,
    is_visible: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct RowModelIdSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CoreRowsSnapshot {
    core: RowModelIdSnapshot,
    row_model: RowModelIdSnapshot,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CoreModelExpect {
    schema_version: u32,
    column_tree: Vec<ColumnNodeSnapshot>,
    column_capabilities: BTreeMap<String, ColumnCapabilitySnapshot>,
    leaf_columns: LeafColumnsSnapshot,
    header_groups: Vec<HeaderGroupSnapshot>,
    left_header_groups: Vec<HeaderGroupSnapshot>,
    center_header_groups: Vec<HeaderGroupSnapshot>,
    right_header_groups: Vec<HeaderGroupSnapshot>,
    rows: CoreRowsSnapshot,
    cells: BTreeMap<String, RowCellsSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    headers_cells: HeadersCellsExpect,
    core_model: CoreModelExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    #[serde(default)]
    state: serde_json::Value,
    expect: FixtureExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

fn header_groups_to_jsonish(
    groups: Vec<fret_ui_headless::table::HeaderGroupSnapshot>,
) -> Vec<HeaderGroupSnapshot> {
    groups
        .into_iter()
        .map(|g| HeaderGroupSnapshot {
            id: g.id.as_ref().to_string(),
            depth: g.depth,
            headers: g.headers.into_iter().map(header_to_jsonish).collect(),
        })
        .collect()
}

fn headers_to_jsonish(
    headers: Vec<fret_ui_headless::table::HeaderSnapshot>,
) -> Vec<HeaderSnapshot> {
    headers.into_iter().map(header_to_jsonish).collect()
}

fn cells_to_jsonish(cells: fret_ui_headless::table::RowCellsSnapshot) -> RowCellsSnapshot {
    let conv = |c: fret_ui_headless::table::CellSnapshot| CellSnapshot {
        id: c.id.as_ref().to_string(),
        column_id: c.column_id.as_ref().to_string(),
        is_grouped: c.is_grouped,
        is_placeholder: c.is_placeholder,
        is_aggregated: c.is_aggregated,
    };
    RowCellsSnapshot {
        all: cells.all.into_iter().map(conv).collect(),
        visible: cells.visible.into_iter().map(conv).collect(),
        left: cells.left.into_iter().map(conv).collect(),
        center: cells.center.into_iter().map(conv).collect(),
        right: cells.right.into_iter().map(conv).collect(),
    }
}

fn core_model_to_jsonish(snapshot: fret_ui_headless::table::CoreModelSnapshot) -> CoreModelExpect {
    let conv_pin = |p: fret_ui_headless::table::ColumnPinPosition| match p {
        fret_ui_headless::table::ColumnPinPosition::Left => ColumnPinPosition::Left,
        fret_ui_headless::table::ColumnPinPosition::Right => ColumnPinPosition::Right,
    };

    CoreModelExpect {
        schema_version: snapshot.schema_version,
        column_tree: snapshot
            .column_tree
            .into_iter()
            .map(|n| ColumnNodeSnapshot {
                id: n.id.as_ref().to_string(),
                depth: n.depth,
                parent_id: n.parent_id.as_ref().map(|s| s.as_ref().to_string()),
                child_ids: n
                    .child_ids
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
            })
            .collect(),
        column_capabilities: snapshot
            .column_capabilities
            .into_iter()
            .map(|(k, v)| {
                (
                    k.as_ref().to_string(),
                    ColumnCapabilitySnapshot {
                        can_hide: v.can_hide,
                        can_pin: v.can_pin,
                        pin_position: v.pin_position.map(conv_pin),
                        pinned_index: v.pinned_index,
                        can_resize: v.can_resize,
                        is_visible: v.is_visible,
                    },
                )
            })
            .collect(),
        leaf_columns: LeafColumnsSnapshot {
            all: snapshot
                .leaf_columns
                .all
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
            visible: snapshot
                .leaf_columns
                .visible
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
            left_visible: snapshot
                .leaf_columns
                .left_visible
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
            center_visible: snapshot
                .leaf_columns
                .center_visible
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
            right_visible: snapshot
                .leaf_columns
                .right_visible
                .into_iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        },
        header_groups: header_groups_to_jsonish(snapshot.header_groups),
        left_header_groups: header_groups_to_jsonish(snapshot.left_header_groups),
        center_header_groups: header_groups_to_jsonish(snapshot.center_header_groups),
        right_header_groups: header_groups_to_jsonish(snapshot.right_header_groups),
        rows: CoreRowsSnapshot {
            core: RowModelIdSnapshot {
                root: snapshot
                    .rows
                    .core
                    .root
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
                flat: snapshot
                    .rows
                    .core
                    .flat
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
            },
            row_model: RowModelIdSnapshot {
                root: snapshot
                    .rows
                    .row_model
                    .root
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
                flat: snapshot
                    .rows
                    .row_model
                    .flat
                    .into_iter()
                    .map(|s| s.as_ref().to_string())
                    .collect(),
            },
        },
        cells: snapshot
            .cells
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), cells_to_jsonish(v)))
            .collect(),
    }
}

#[test]
fn tanstack_v8_headers_cells_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("headers_cells.json");

    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "headers_cells");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name"),
        ColumnDef::<FixtureRow>::new("stats").columns(vec![
            ColumnDef::<FixtureRow>::new("perf").columns(vec![ColumnDef::<FixtureRow>::new("cpu")]),
            ColumnDef::<FixtureRow>::new("mem")
                .columns(vec![ColumnDef::<FixtureRow>::new("mem_mb")]),
        ]),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let state = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .build();

        assert_eq!(
            header_groups_to_jsonish(table.header_groups()),
            snap.expect.headers_cells.header_groups,
            "snapshot {} header_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.footer_groups()),
            snap.expect.headers_cells.footer_groups,
            "snapshot {} footer_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.left_header_groups()),
            snap.expect.headers_cells.left_header_groups,
            "snapshot {} left_header_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.left_footer_groups()),
            snap.expect.headers_cells.left_footer_groups,
            "snapshot {} left_footer_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.center_header_groups()),
            snap.expect.headers_cells.center_header_groups,
            "snapshot {} center_header_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.center_footer_groups()),
            snap.expect.headers_cells.center_footer_groups,
            "snapshot {} center_footer_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.right_header_groups()),
            snap.expect.headers_cells.right_header_groups,
            "snapshot {} right_header_groups mismatch",
            snap.id
        );
        assert_eq!(
            header_groups_to_jsonish(table.right_footer_groups()),
            snap.expect.headers_cells.right_footer_groups,
            "snapshot {} right_footer_groups mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.flat_headers()),
            snap.expect.headers_cells.flat_headers,
            "snapshot {} flat_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.left_flat_headers()),
            snap.expect.headers_cells.left_flat_headers,
            "snapshot {} left_flat_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.center_flat_headers()),
            snap.expect.headers_cells.center_flat_headers,
            "snapshot {} center_flat_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.right_flat_headers()),
            snap.expect.headers_cells.right_flat_headers,
            "snapshot {} right_flat_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.leaf_headers()),
            snap.expect.headers_cells.leaf_headers,
            "snapshot {} leaf_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.left_leaf_headers()),
            snap.expect.headers_cells.left_leaf_headers,
            "snapshot {} left_leaf_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.center_leaf_headers()),
            snap.expect.headers_cells.center_leaf_headers,
            "snapshot {} center_leaf_headers mismatch",
            snap.id
        );
        assert_eq!(
            headers_to_jsonish(table.right_leaf_headers()),
            snap.expect.headers_cells.right_leaf_headers,
            "snapshot {} right_leaf_headers mismatch",
            snap.id
        );

        for (row_id, expected_cells) in &snap.expect.headers_cells.cells {
            let row_key = RowKey(row_id.parse::<u64>().expect("row id as u64"));
            let got = table.row_cells(row_key).expect("row cells");
            assert_eq!(
                cells_to_jsonish(got),
                expected_cells.clone(),
                "snapshot {} cells mismatch for row {}",
                snap.id,
                row_id
            );
        }

        assert_eq!(
            core_model_to_jsonish(table.core_model_snapshot()),
            snap.expect.core_model,
            "snapshot {} core_model mismatch",
            snap.id
        );
    }
}
