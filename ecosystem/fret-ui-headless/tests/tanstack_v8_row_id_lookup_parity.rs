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
struct RowsByIdExpect {
    core: Vec<String>,
    pre_pagination: Vec<String>,
    row_model: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct LookupExpect {
    row_id: String,
    search_all: bool,
    found: bool,
    #[serde(default)]
    found_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    rows_by_id: RowsByIdExpect,
    lookups: Vec<LookupExpect>,
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

fn row_model_rows_by_id<'a, TData>(
    model: &fret_ui_headless::table::RowModel<'a, TData>,
) -> Vec<String> {
    let mut ids = model
        .rows_by_id()
        .keys()
        .map(|id| id.as_str().to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

#[test]
fn tanstack_v8_row_id_lookup_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("row_id_lookup.json");

    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "row_id_lookup");

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

        let mut builder = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .state(state)
            .options(options);

        if snap
            .options
            .get("__getRowId")
            .and_then(|v| v.as_str())
            .is_some_and(|mode| mode == "prefixed")
        {
            builder =
                builder.get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)));
        } else {
            builder = builder.get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()));
        }

        let table = builder.build();

        assert_eq!(
            row_model_rows_by_id(table.core_row_model()),
            snap.expect.rows_by_id.core,
            "snapshot {} core rows_by_id mismatch",
            snap.id
        );
        assert_eq!(
            row_model_rows_by_id(table.pre_pagination_row_model()),
            snap.expect.rows_by_id.pre_pagination,
            "snapshot {} pre_pagination rows_by_id mismatch",
            snap.id
        );
        assert_eq!(
            row_model_rows_by_id(table.row_model()),
            snap.expect.rows_by_id.row_model,
            "snapshot {} row_model rows_by_id mismatch",
            snap.id
        );

        for lookup in &snap.expect.lookups {
            let actual = table.row_by_id(lookup.row_id.as_str(), lookup.search_all);
            assert_eq!(
                actual.is_some(),
                lookup.found,
                "snapshot {} lookup found mismatch for row_id={} search_all={}",
                snap.id,
                lookup.row_id,
                lookup.search_all
            );
            let actual_id = actual.map(|row| row.id.as_str().to_string());
            assert_eq!(
                actual_id, lookup.found_id,
                "snapshot {} lookup found_id mismatch for row_id={} search_all={}",
                snap.id, lookup.row_id, lookup.search_all
            );
        }
    }
}
