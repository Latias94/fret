use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    BuiltInAggregationFn, ColumnDef, GroupedRowKind, GroupedRowModel, RowId, RowKey, Table,
    TanStackTableOptions, TanStackTableState, TanStackValue,
};
use serde::{Deserialize, de::Deserializer};

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: u64,
    team: String,
    score: u64,
    #[serde(default, deserialize_with = "deserialize_js_value")]
    tag: JsValue,
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
struct PathEntry {
    column_id: String,
    value: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct GroupedAggregationsAnyExpect {
    path: Vec<PathEntry>,
    values: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    #[serde(default)]
    grouped_aggregations_any: Option<Vec<GroupedAggregationsAnyExpect>>,
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

fn tanstack_value_num(v: u64) -> TanStackValue {
    TanStackValue::Number(v as f64)
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

fn group_nodes_with_path(model: &GroupedRowModel) -> Vec<(RowKey, Vec<PathEntry>)> {
    fn walk(
        model: &GroupedRowModel,
        index: usize,
        path: &[PathEntry],
        out: &mut Vec<(RowKey, Vec<PathEntry>)>,
    ) {
        let Some(row) = model.row(index) else {
            return;
        };

        let (next_path, is_group) = match &row.kind {
            GroupedRowKind::Group {
                grouping_column,
                grouping_value,
                ..
            } => {
                let mut next = path.to_vec();
                next.push(PathEntry {
                    column_id: grouping_column.to_string(),
                    value: serde_json::Value::String(grouping_value.to_string()),
                });
                (next, true)
            }
            GroupedRowKind::Leaf { .. } => (path.to_vec(), false),
        };

        if is_group {
            out.push((row.key, next_path.clone()));
        }

        for &child in &row.sub_rows {
            walk(model, child, next_path.as_slice(), out);
        }
    }

    let mut out = Vec::new();
    for &root in model.root_rows() {
        walk(model, root, &[], &mut out);
    }
    out
}

fn path_key(path: &[PathEntry]) -> String {
    path.iter()
        .map(|e| {
            let v = e
                .value
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| e.value.to_string());
            format!("{}={}", e.column_id, v)
        })
        .collect::<Vec<_>>()
        .join("|")
}

#[test]
fn tanstack_v8_grouping_aggregation_fns_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("grouping_aggregation_fns.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "grouping_aggregation_fns");

    let data = fixture.data;

    let score_value = |row: &FixtureRow| tanstack_value_num(row.score);
    let tag_value = |row: &FixtureRow| tanstack_value_from_js_value(&row.tag);

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("role").facet_key_by(|row: &FixtureRow| row.role),
        ColumnDef::<FixtureRow>::new("team")
            .sort_value_by(|row: &FixtureRow| tanstack_value_str(&row.team)),
        ColumnDef::<FixtureRow>::new("score_sum")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Sum),
        ColumnDef::<FixtureRow>::new("score_min")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Min),
        ColumnDef::<FixtureRow>::new("score_max")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Max),
        ColumnDef::<FixtureRow>::new("score_extent")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Extent),
        ColumnDef::<FixtureRow>::new("score_mean")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Mean),
        ColumnDef::<FixtureRow>::new("score_median")
            .sort_value_by(score_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Median),
        ColumnDef::<FixtureRow>::new("tag_unique")
            .sort_value_by(tag_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Unique),
        ColumnDef::<FixtureRow>::new("tag_unique_count")
            .sort_value_by(tag_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::UniqueCount),
        ColumnDef::<FixtureRow>::new("tag_count")
            .sort_value_by(tag_value)
            .aggregation_fn_builtin(BuiltInAggregationFn::Count),
        ColumnDef::<FixtureRow>::new("score_custom")
            .sort_value_by(score_value)
            .aggregation_fn_named("custom_plus_one"),
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
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options);

        let aggregation_fns_mode = snap
            .options
            .get("aggregationFnsMode")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if aggregation_fns_mode == "custom_plus_one" {
            builder = builder.aggregation_fn(
                "custom_plus_one",
                Arc::new(|_column_id, values| {
                    let mut sum = 0.0;
                    for v in values {
                        let n = match v {
                            TanStackValue::Number(n) => Some(*n),
                            TanStackValue::DateTime(n) => Some(*n),
                            _ => None,
                        };
                        let Some(n) = n else {
                            continue;
                        };
                        if n.is_nan() {
                            continue;
                        }
                        sum += n;
                    }
                    TanStackValue::Number(sum + 1.0)
                }),
            );
        }

        let table = builder.build();

        let expected = snap.expect.grouped_aggregations_any.unwrap_or_default();
        let mut expected = expected;
        expected.sort_by_key(|e| path_key(&e.path));

        let grouped = table.grouped_row_model();
        let actual_aggs = table.grouped_aggregations_any();

        let mut actual_entries: Vec<GroupedAggregationsAnyExpect> = Vec::new();
        for (key, path) in group_nodes_with_path(grouped) {
            let mut values: BTreeMap<String, serde_json::Value> = BTreeMap::new();
            if let Some(entries) = actual_aggs.get(&key) {
                for (col_id, v) in entries.iter() {
                    values.insert(col_id.to_string(), json_safe_value(v));
                }
            }
            actual_entries.push(GroupedAggregationsAnyExpect { path, values });
        }
        actual_entries.sort_by_key(|e| path_key(&e.path));

        assert_eq!(
            actual_entries, expected,
            "snapshot {} grouped_aggregations_any mismatch",
            snap.id
        );
    }
}
