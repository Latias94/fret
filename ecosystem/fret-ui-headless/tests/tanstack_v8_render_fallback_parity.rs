use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, RowKey, Table, TanStackTableOptions, TanStackTableState, TanStackValue,
};
use serde::{Deserialize, de::Deserializer};

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[serde(default, deserialize_with = "deserialize_js_value")]
    value: JsValue,
}

#[derive(Debug, Clone)]
enum JsValue {
    Undefined,
    Null,
    Value(serde_json::Value),
}

impl Default for JsValue {
    fn default() -> Self {
        Self::Undefined
    }
}

fn deserialize_js_value<'de, D>(deserializer: D) -> Result<JsValue, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(match opt {
        None => JsValue::Null,
        Some(v) => JsValue::Value(v),
    })
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RenderFallbackEntryExpect {
    row_id: String,
    column_id: String,
    value: serde_json::Value,
    render_value: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    render_fallback: Vec<RenderFallbackEntryExpect>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    state: serde_json::Value,
    expect: FixtureExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

fn json_undefined() -> serde_json::Value {
    serde_json::json!({ "__fret": "undefined" })
}

fn json_number(n: f64) -> serde_json::Value {
    if !n.is_finite() {
        return serde_json::Value::Null;
    }
    if n.fract() == 0.0 && (i64::MIN as f64) <= n && n <= (i64::MAX as f64) {
        return serde_json::Value::Number(serde_json::Number::from(n as i64));
    }
    serde_json::Number::from_f64(n)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

fn json_safe_value(value: &TanStackValue) -> serde_json::Value {
    match value {
        TanStackValue::Undefined => json_undefined(),
        TanStackValue::Null => serde_json::Value::Null,
        TanStackValue::Bool(b) => serde_json::Value::Bool(*b),
        TanStackValue::Number(n) | TanStackValue::DateTime(n) => json_number(*n),
        TanStackValue::String(s) => serde_json::Value::String(s.to_string()),
        TanStackValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(json_safe_value).collect())
        }
    }
}

fn tanstack_value_str(s: &str) -> TanStackValue {
    TanStackValue::String(Arc::<str>::from(s))
}

fn tanstack_value_from_json(value: &serde_json::Value) -> TanStackValue {
    match value {
        serde_json::Value::Null => TanStackValue::Null,
        serde_json::Value::Bool(b) => TanStackValue::Bool(*b),
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(TanStackValue::Number)
            .unwrap_or(TanStackValue::Undefined),
        serde_json::Value::String(s) => tanstack_value_str(s),
        serde_json::Value::Array(arr) => {
            TanStackValue::Array(arr.iter().map(tanstack_value_from_json).collect())
        }
        serde_json::Value::Object(map) => {
            if map.get("__fret").and_then(|v| v.as_str()) == Some("undefined") {
                TanStackValue::Undefined
            } else {
                TanStackValue::Undefined
            }
        }
    }
}

fn tanstack_value_from_js_value(value: &JsValue) -> TanStackValue {
    match value {
        JsValue::Undefined => TanStackValue::Undefined,
        JsValue::Null => TanStackValue::Null,
        JsValue::Value(v) => tanstack_value_from_json(v),
    }
}

#[test]
fn tanstack_v8_render_fallback_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("render_fallback.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "render_fallback");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("value")
            .sort_value_by(|row: &FixtureRow| tanstack_value_from_js_value(&row.value)),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let state = tanstack_state.to_table_state().expect("state conversion");

        let fallback = snap
            .options
            .get("renderFallbackValue")
            .map(tanstack_value_from_json)
            .unwrap_or(TanStackValue::Null);

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .state(state)
            .options(options)
            .render_fallback_value(fallback)
            .build();

        let mut actual = Vec::new();
        for &index in table.core_row_model().flat_rows() {
            let row = table.core_row_model().row(index).expect("row");
            let row_id = row.key.0.to_string();
            let value = table
                .cell_value(row.key, "value")
                .unwrap_or(TanStackValue::Undefined);
            let render_value = table
                .cell_render_value(row.key, "value")
                .unwrap_or(TanStackValue::Undefined);
            actual.push(RenderFallbackEntryExpect {
                row_id,
                column_id: "value".to_string(),
                value: json_safe_value(&value),
                render_value: json_safe_value(&render_value),
            });
        }

        assert_eq!(
            actual, snap.expect.render_fallback,
            "snapshot {} render_fallback mismatch",
            snap.id
        );
    }
}
