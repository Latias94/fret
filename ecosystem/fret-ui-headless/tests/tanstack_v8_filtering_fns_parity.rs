use std::path::PathBuf;

use fret_ui_headless::table::{
    BuiltInFilterFn, ColumnDef, FilteringFnSpec, RowId, RowKey, Table, TanStackTableOptions,
    TanStackTableState, TanStackValue, set_column_filter_value_tanstack,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[serde(default)]
    text: Option<String>,
    num: u64,
    flag: bool,
    tags: Vec<String>,
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
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "setColumnFilterValue")]
    SetColumnFilterValue {
        column_id: String,
        value: serde_json::Value,
    },
    #[serde(rename = "setGlobalFilterValue")]
    SetGlobalFilterValue { value: serde_json::Value },
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    #[serde(default)]
    state: serde_json::Value,
    #[serde(default)]
    actions: Vec<FixtureAction>,
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
fn tanstack_v8_filtering_fns_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("filtering_fns.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "filtering_fns");

    let data = fixture.data;

    let text_value = |row: &FixtureRow| match row.text.as_ref() {
        Some(s) => TanStackValue::String(std::sync::Arc::<str>::from(s.as_str())),
        None => TanStackValue::Null,
    };
    let num_value = |row: &FixtureRow| TanStackValue::Number(row.num as f64);
    let flag_value = |row: &FixtureRow| TanStackValue::Bool(row.flag);
    let tags_value = |row: &FixtureRow| {
        TanStackValue::Array(
            row.tags
                .iter()
                .map(|t| TanStackValue::String(std::sync::Arc::<str>::from(t.as_str())))
                .collect(),
        )
    };

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("text_auto")
            .sort_value_by(text_value)
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("text_equals_string")
            .sort_value_by(text_value)
            .filtering_fn_named("equalsString"),
        ColumnDef::<FixtureRow>::new("num_range")
            .sort_value_by(num_value)
            .filtering_fn_named("inNumberRange"),
        ColumnDef::<FixtureRow>::new("num_weak")
            .sort_value_by(num_value)
            .filtering_fn_named("weakEquals"),
        ColumnDef::<FixtureRow>::new("tags_all")
            .sort_value_by(tags_value)
            .filtering_fn_named("arrIncludesAll"),
        ColumnDef::<FixtureRow>::new("flag_equals")
            .sort_value_by(flag_value)
            .filtering_fn_named("equals"),
        ColumnDef::<FixtureRow>::new("text_custom")
            .sort_value_by(text_value)
            .filtering_fn_named("custom_text"),
    ];

    let column_by_id: std::collections::HashMap<&str, &ColumnDef<FixtureRow>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    let mut filter_fns: std::collections::HashMap<
        std::sync::Arc<str>,
        fret_ui_headless::table::FilterFnDef,
    > = std::collections::HashMap::new();
    filter_fns.insert(
        std::sync::Arc::<str>::from("custom_text"),
        fret_ui_headless::table::FilterFnDef::BuiltIn(BuiltInFilterFn::IncludesStringSensitive),
    );

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        if !snap.actions.is_empty() {
            let on_column_filters_change_mode = snap
                .options
                .get("__onColumnFiltersChange")
                .and_then(|v| v.as_str());
            let on_global_filter_change_mode = snap
                .options
                .get("__onGlobalFilterChange")
                .and_then(|v| v.as_str());

            for action in &snap.actions {
                match action {
                    FixtureAction::SetColumnFilterValue { column_id, value } => {
                        if on_column_filters_change_mode == Some("noop") {
                            continue;
                        }
                        let Some(column) = column_by_id.get(column_id.as_str()).copied() else {
                            panic!("unknown action column_id: {}", column_id);
                        };
                        set_column_filter_value_tanstack(
                            &mut state.column_filters,
                            &data,
                            column,
                            &filter_fns,
                            value.clone(),
                        );
                    }
                    FixtureAction::SetGlobalFilterValue { value } => {
                        if on_global_filter_change_mode == Some("noop") {
                            continue;
                        }
                        state.global_filter = Some(value.clone());
                    }
                }
            }

            if let Some(expected_next) = snap.expect.next_state.as_ref() {
                let tanstack_next =
                    TanStackTableState::from_json(expected_next).expect("tanstack next_state");
                let expected_state = tanstack_next
                    .to_table_state()
                    .expect("next_state conversion");
                assert_eq!(
                    state.column_filters, expected_state.column_filters,
                    "snapshot {} next_state.column_filters mismatch",
                    snap.id
                );
                assert_eq!(
                    state.global_filter, expected_state.global_filter,
                    "snapshot {} next_state.global_filter mismatch",
                    snap.id
                );
            }
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .filter_fn_builtin("custom_text", BuiltInFilterFn::IncludesStringSensitive)
            .global_filter_fn(FilteringFnSpec::Auto)
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
