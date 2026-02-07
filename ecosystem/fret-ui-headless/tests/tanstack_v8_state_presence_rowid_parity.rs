use std::path::PathBuf;

use fret_ui_headless::table::{
    Aggregation, ColumnDef, ExpandingState, RowId, RowKey, Table, TanStackTableState,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: String,
    team: String,
    score: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct SnapshotExpect {
    json_roundtrip: serde_json::Value,
    state_row_selection_len: usize,
    state_expanded_kind: String,
    state_expanded_keys_len: usize,
    state_row_pinning_top_len: usize,
    state_row_pinning_bottom_len: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    #[serde(default)]
    state: serde_json::Value,
    expect: SnapshotExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

#[test]
fn tanstack_v8_state_presence_rowid_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("state_presence_rowid.json");

    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "state_presence_rowid");

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("role")
            .facet_str_by(|row: &FixtureRow| row.role.as_str())
            .aggregate(Aggregation::Count),
        ColumnDef::<FixtureRow>::new("team")
            .facet_str_by(|row: &FixtureRow| row.team.as_str())
            .aggregate(Aggregation::Count),
        ColumnDef::<FixtureRow>::new("score")
            .value_u64_by(|row: &FixtureRow| row.score)
            .aggregate(Aggregation::SumU64),
    ];

    for snap in fixture.snapshots {
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");

        let mut base_state_src = tanstack_state.clone();
        base_state_src.expanded = None;
        base_state_src.row_pinning = None;
        base_state_src.row_selection = None;
        let base_state = base_state_src.to_table_state().expect("base state");

        let initial_table = Table::builder(&fixture.data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)))
            .state(base_state)
            .build();

        let state = tanstack_state
            .to_table_state_with_row_models(
                initial_table.core_row_model(),
                Some(initial_table.grouped_row_model()),
            )
            .expect("to_table_state_with_row_models");

        assert_eq!(
            state.row_selection.len(),
            snap.expect.state_row_selection_len,
            "snapshot {} row_selection len mismatch",
            snap.id
        );

        match &state.expanding {
            ExpandingState::All => {
                assert_eq!(
                    snap.expect.state_expanded_kind, "all",
                    "snapshot {} expanded kind mismatch",
                    snap.id
                );
            }
            ExpandingState::Keys(keys) => {
                assert_eq!(
                    snap.expect.state_expanded_kind, "keys",
                    "snapshot {} expanded kind mismatch",
                    snap.id
                );
                assert_eq!(
                    keys.len(),
                    snap.expect.state_expanded_keys_len,
                    "snapshot {} expanded keys len mismatch",
                    snap.id
                );
            }
        }

        assert_eq!(
            state.row_pinning.top.len(),
            snap.expect.state_row_pinning_top_len,
            "snapshot {} row_pinning.top len mismatch",
            snap.id
        );
        assert_eq!(
            state.row_pinning.bottom.len(),
            snap.expect.state_row_pinning_bottom_len,
            "snapshot {} row_pinning.bottom len mismatch",
            snap.id
        );

        let runtime_table = Table::builder(&fixture.data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)))
            .state(state.clone())
            .build();

        let roundtrip = TanStackTableState::from_table_state_with_row_models_and_shape(
            &state,
            runtime_table.core_row_model(),
            Some(runtime_table.grouped_row_model()),
            &tanstack_state,
        )
        .expect("from_table_state_with_row_models_and_shape");

        let actual_json = roundtrip.to_json().expect("to_json");
        assert_eq!(
            actual_json, snap.expect.json_roundtrip,
            "snapshot {} json roundtrip mismatch",
            snap.id
        );
    }
}
