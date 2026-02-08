use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, RowModel, Table, TanStackTableOptions, TanStackTableState,
    TanStackValue,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RowFilterStateSnapshotExpect {
    filterable_ids: Vec<String>,
    row_column_filters: HashMap<String, HashMap<String, bool>>,
    row_column_filters_meta: HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    row_filter_state: RowFilterStateSnapshotExpect,
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

#[derive(Debug, Clone, PartialEq)]
struct RowFilterStateSnapshotNormalized {
    filterable_ids: Vec<String>,
    row_column_filters: BTreeMap<String, BTreeMap<String, bool>>,
    row_column_filters_meta: BTreeMap<String, BTreeMap<String, Value>>,
}

fn snapshot_row_model<'a, TData>(model: &RowModel<'a, TData>) -> RowModelSnapshot {
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

fn normalize_row_filter_state(
    snapshot: &fret_ui_headless::table::RowFilterStateSnapshot,
) -> RowFilterStateSnapshotNormalized {
    let mut row_column_filters = BTreeMap::new();
    for (row_key, filters) in &snapshot.row_column_filters {
        let mut normalized_filters = BTreeMap::new();
        for (column_id, pass) in filters {
            normalized_filters.insert(column_id.as_ref().to_string(), *pass);
        }
        row_column_filters.insert(row_key.0.to_string(), normalized_filters);
    }

    let mut row_column_filters_meta = BTreeMap::new();
    for (row_key, meta_map) in &snapshot.row_column_filters_meta {
        let mut normalized_meta = BTreeMap::new();
        for (column_id, meta) in meta_map {
            normalized_meta.insert(column_id.as_ref().to_string(), meta.clone());
        }
        row_column_filters_meta.insert(row_key.0.to_string(), normalized_meta);
    }

    RowFilterStateSnapshotNormalized {
        filterable_ids: snapshot
            .filterable_ids
            .iter()
            .map(|id| id.as_ref().to_string())
            .collect(),
        row_column_filters,
        row_column_filters_meta,
    }
}

fn normalize_expected_row_filter_state(
    expected: &RowFilterStateSnapshotExpect,
) -> RowFilterStateSnapshotNormalized {
    let row_column_filters = expected
        .row_column_filters
        .iter()
        .map(|(row_key, filters)| {
            (
                row_key.clone(),
                filters
                    .iter()
                    .map(|(column_id, pass)| (column_id.clone(), *pass))
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let row_column_filters_meta = expected
        .row_column_filters_meta
        .iter()
        .map(|(row_key, meta_map)| {
            (
                row_key.clone(),
                meta_map
                    .iter()
                    .map(|(column_id, meta)| (column_id.clone(), meta.clone()))
                    .collect::<BTreeMap<_, _>>(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    RowFilterStateSnapshotNormalized {
        filterable_ids: expected.filterable_ids.clone(),
        row_column_filters,
        row_column_filters_meta,
    }
}

#[test]
fn tanstack_v8_filtering_meta_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("filtering_meta.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "filtering_meta");

    let data = fixture.data;
    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("text_meta")
            .sort_value_by(|row| {
                TanStackValue::String(std::sync::Arc::<str>::from(row.text.as_str()))
            })
            .filtering_fn_named("meta_starts_with"),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let state = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .filter_fn_value_with_meta("meta_starts_with", |value, filter_value, add_meta| {
                let cell = match value {
                    TanStackValue::String(text) => text.as_ref(),
                    _ => "",
                };
                let needle = filter_value.as_str().unwrap_or_default();
                let pass = cell.starts_with(needle);
                add_meta(json!({ "value": cell, "pass": pass, "len": cell.len() }));
                pass
            })
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .build();

        let core = snapshot_row_model(table.core_row_model());
        let filtered = snapshot_row_model(table.filtered_row_model());
        let sorted = snapshot_row_model(table.sorted_row_model());
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

        let actual_row_filter_state =
            normalize_row_filter_state(&table.row_filter_state_snapshot());
        let expected_row_filter_state =
            normalize_expected_row_filter_state(&snap.expect.row_filter_state);
        assert_eq!(
            actual_row_filter_state, expected_row_filter_state,
            "snapshot {} row_filter_state mismatch",
            snap.id
        );
    }
}
