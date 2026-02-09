use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    BuiltInSortingFn, ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
    TanStackValue,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[serde(flatten)]
    fields: HashMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: Value,
    #[serde(default)]
    state: Value,
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

fn value_from_json(v: Option<&Value>) -> TanStackValue {
    let Some(v) = v else {
        return TanStackValue::Undefined;
    };
    match v {
        Value::Null => TanStackValue::Null,
        Value::Bool(b) => TanStackValue::Bool(*b),
        Value::Number(n) => TanStackValue::Number(n.as_f64().unwrap_or(f64::NAN)),
        Value::String(s) => TanStackValue::String(Arc::<str>::from(s.as_str())),
        Value::Array(arr) => {
            TanStackValue::Array(arr.iter().map(|v| value_from_json(Some(v))).collect())
        }
        Value::Object(_) => TanStackValue::Undefined,
    }
}

fn text_value_from_json(v: Option<&Value>) -> TanStackValue {
    let Some(v) = v else {
        return TanStackValue::Undefined;
    };
    match v {
        Value::String(s) if s == "__NaN__" => TanStackValue::Number(f64::NAN),
        Value::String(s) if s == "__Inf__" => TanStackValue::Number(f64::INFINITY),
        Value::String(s) if s == "__-Inf__" => TanStackValue::Number(f64::NEG_INFINITY),
        _ => value_from_json(Some(v)),
    }
}

fn dt_value_from_json(v: Option<&Value>) -> TanStackValue {
    let Some(v) = v else {
        return TanStackValue::Undefined;
    };
    match v {
        Value::Null => TanStackValue::Null,
        Value::Number(n) => TanStackValue::DateTime(n.as_f64().unwrap_or(f64::NAN)),
        _ => TanStackValue::Undefined,
    }
}

#[test]
fn tanstack_v8_sorting_edge_cases_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("sorting_edge_cases.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "sorting_edge_cases");

    let data = fixture.data;

    let text_value = |row: &FixtureRow| text_value_from_json(row.fields.get("text_raw"));
    let alpha_value = |row: &FixtureRow| match row.fields.get("alpha") {
        Some(Value::String(s)) => TanStackValue::String(Arc::<str>::from(s.as_str())),
        Some(Value::Null) => TanStackValue::Null,
        Some(_) => TanStackValue::Undefined,
        None => TanStackValue::Undefined,
    };
    let dt_value = |row: &FixtureRow| dt_value_from_json(row.fields.get("dt_ms"));

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("text_mixed_default")
            .sort_value_by(text_value)
            .sorting_fn_named("text"),
        ColumnDef::<FixtureRow>::new("text_mixed_false")
            .sort_value_by(text_value)
            .sorting_fn_named("text")
            .sort_undefined_disabled(),
        ColumnDef::<FixtureRow>::new("alpha_mixed")
            .sort_value_by(alpha_value)
            .sorting_fn_named("alphanumeric"),
        ColumnDef::<FixtureRow>::new("dt_mixed")
            .sort_value_by(dt_value)
            .sorting_fn_builtin(BuiltInSortingFn::Datetime),
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
    }
}
