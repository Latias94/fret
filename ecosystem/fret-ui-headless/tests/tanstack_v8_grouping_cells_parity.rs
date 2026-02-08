use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    Aggregation, ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: u64,
    team: u64,
    score: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct CellSnapshot {
    id: String,
    column_id: String,
    is_grouped: bool,
    is_placeholder: bool,
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
struct FixtureExpect {
    cells: BTreeMap<String, RowCellsSnapshot>,
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

#[test]
fn tanstack_v8_grouping_cells_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("grouping_cells.json");

    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "grouping_cells");

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
        let state = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .build();

        assert_eq!(
            table.state().grouping.len(),
            2,
            "snapshot {} expected two-level grouping",
            snap.id
        );

        for (row_id, expected_cells) in &snap.expect.cells {
            let row_key = table
                .row_key_for_id(row_id, true)
                .unwrap_or_else(|| panic!("snapshot {} missing row id {}", snap.id, row_id));
            let got = table
                .row_cells(row_key)
                .unwrap_or_else(|| panic!("snapshot {} missing row cells for {}", snap.id, row_id));
            assert_eq!(
                cells_to_jsonish(got),
                expected_cells.clone(),
                "snapshot {} cells mismatch for row {}",
                snap.id,
                row_id
            );
        }
    }
}
