use std::path::PathBuf;

use fret_ui_headless::table::{ColumnDef, RowId, RowKey, SortUndefined, Table, TanStackValue};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[serde(default)]
    rank: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    sorted: RowModelSnapshot,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
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
fn tanstack_v8_sort_undefined_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("sort_undefined.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "sort_undefined");

    let data = fixture.data;

    let rank_cmp = |a: &FixtureRow, b: &FixtureRow| match (a.rank, b.rank) {
        (Some(a), Some(b)) => a.cmp(&b),
        _ => std::cmp::Ordering::Equal,
    };

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("rank_first")
            .sort_by(rank_cmp)
            .sort_undefined_by(SortUndefined::First, |row| row.rank.is_none()),
        ColumnDef::<FixtureRow>::new("rank_last")
            .sort_by(rank_cmp)
            .sort_undefined_by(SortUndefined::Last, |row| row.rank.is_none()),
        ColumnDef::<FixtureRow>::new("rank_1")
            .sort_by(rank_cmp)
            .sort_undefined_by(SortUndefined::Dir(1), |row| row.rank.is_none()),
        ColumnDef::<FixtureRow>::new("rank_neg1")
            .sort_by(rank_cmp)
            .sort_undefined_by(SortUndefined::Dir(-1), |row| row.rank.is_none()),
        ColumnDef::<FixtureRow>::new("rank_false_text")
            .sort_value_by(|row| match row.rank {
                Some(v) => TanStackValue::Number(v as f64),
                None => TanStackValue::Undefined,
            })
            .sorting_fn_named("text")
            .sort_undefined_disabled(),
    ];

    for snap in fixture.snapshots {
        let tanstack_state =
            fret_ui_headless::table::TanStackTableState::from_json(&snap.state).expect("state");
        let state = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .build();

        let sorted = snapshot_row_model(table.sorted_row_model());
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
    }
}
