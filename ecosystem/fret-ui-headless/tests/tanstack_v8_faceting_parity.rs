use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TableState, TanStackTableOptions, TanStackTableState,
    TanStackValue,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    status: String,
    cpu: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct FacetingColumnExpect {
    row_model: RowModelSnapshot,
    unique_values: BTreeMap<String, usize>,
    min_max: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct FacetingExpect {
    cpu: FacetingColumnExpect,
    global: FacetingColumnExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    faceting: FacetingExpect,
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
fn tanstack_v8_faceting_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("faceting.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "faceting");

    let data = fixture.data;

    let status = ColumnDef::<FixtureRow>::new("status").filter_by(|row: &FixtureRow, q| {
        row.status
            .to_ascii_lowercase()
            .contains(&q.to_ascii_lowercase())
    });
    let cpu = ColumnDef::<FixtureRow>::new("cpu")
        .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.cpu as f64))
        .filtering_fn_named("inNumberRange")
        .facet_key_by(|row: &FixtureRow| row.cpu);

    let columns = vec![status, cpu];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let state: TableState = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .build();

        let cpu = &snap.expect.faceting.cpu;

        let model = table
            .faceted_row_model("cpu")
            .expect("faceted row model missing for cpu");
        assert_eq!(
            snapshot_row_model(model),
            cpu.row_model,
            "snapshot {} faceting.cpu.row_model mismatch",
            snap.id
        );

        let unique = table
            .faceted_unique_values("cpu")
            .expect("faceted unique values missing for cpu");
        let got_unique: BTreeMap<String, usize> =
            unique.iter().map(|(k, v)| (k.to_string(), *v)).collect();
        assert_eq!(
            got_unique, cpu.unique_values,
            "snapshot {} faceting.cpu.unique_values mismatch",
            snap.id
        );

        let got_min_max = table.faceted_min_max_u64("cpu");
        assert_eq!(
            got_min_max, cpu.min_max,
            "snapshot {} faceting.cpu.min_max mismatch",
            snap.id
        );

        let global = &snap.expect.faceting.global;
        assert_eq!(
            snapshot_row_model(table.filtered_row_model()),
            global.row_model,
            "snapshot {} faceting.global.row_model mismatch",
            snap.id
        );

        let got_global_unique: BTreeMap<String, usize> = BTreeMap::new();
        assert_eq!(
            got_global_unique, global.unique_values,
            "snapshot {} faceting.global.unique_values mismatch",
            snap.id
        );

        let got_global_min_max: Option<(u64, u64)> = None;
        assert_eq!(
            got_global_min_max, global.min_max,
            "snapshot {} faceting.global.min_max mismatch",
            snap.id
        );
    }
}
