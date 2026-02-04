use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowKey, Table, TanStackTableOptions, TanStackTableState, TanStackValue,
    toggle_sorting_tanstack,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    text: String,
    alpha: String,
    num: u64,
    #[serde(default)]
    dt_ms: Option<u64>,
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
    #[serde(rename = "toggleSorting")]
    ToggleSorting {
        column_id: String,
        #[serde(default)]
        multi: bool,
    },
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

fn auto_sort_dir_desc_for_column<TData>(data: &[TData], column: &ColumnDef<TData>) -> bool {
    let Some(first) = data.first() else {
        return false;
    };
    let Some(get_value) = column.sort_value.as_ref() else {
        return false;
    };

    // TanStack `getAutoSortDir`: string => asc, everything else => desc.
    !matches!((get_value)(first), TanStackValue::String(_))
}

#[test]
fn tanstack_v8_sorting_fns_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("sorting_fns.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "sorting_fns");

    let data = fixture.data;

    let num_value = |row: &FixtureRow| TanStackValue::Number(row.num as f64);
    let text_value =
        |row: &FixtureRow| TanStackValue::String(std::sync::Arc::<str>::from(row.text.as_str()));
    let alpha_value =
        |row: &FixtureRow| TanStackValue::String(std::sync::Arc::<str>::from(row.alpha.as_str()));
    let dt_value = |row: &FixtureRow| match row.dt_ms {
        Some(ms) => TanStackValue::DateTime(ms as f64),
        None => TanStackValue::Undefined,
    };

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("num_basic")
            .sort_value_by(num_value)
            .sorting_fn_named("basic"),
        ColumnDef::<FixtureRow>::new("num_auto")
            .sort_value_by(num_value)
            .sorting_fn_auto(),
        ColumnDef::<FixtureRow>::new("text_text")
            .sort_value_by(text_value)
            .sorting_fn_named("text"),
        ColumnDef::<FixtureRow>::new("text_text_cs")
            .sort_value_by(text_value)
            .sorting_fn_named("textCaseSensitive"),
        ColumnDef::<FixtureRow>::new("text_auto")
            .sort_value_by(text_value)
            .sorting_fn_auto(),
        ColumnDef::<FixtureRow>::new("alpha_alphanumeric")
            .sort_value_by(alpha_value)
            .sorting_fn_named("alphanumeric"),
        ColumnDef::<FixtureRow>::new("alpha_alphanumeric_cs")
            .sort_value_by(alpha_value)
            .sorting_fn_named("alphanumericCaseSensitive"),
        ColumnDef::<FixtureRow>::new("alpha_auto")
            .sort_value_by(alpha_value)
            .sorting_fn_auto(),
        ColumnDef::<FixtureRow>::new("dt_datetime")
            .sort_value_by(dt_value)
            .sorting_fn_named("datetime"),
        ColumnDef::<FixtureRow>::new("dt_auto")
            .sort_value_by(dt_value)
            .sorting_fn_auto(),
        ColumnDef::<FixtureRow>::new("text_custom")
            .sort_value_by(text_value)
            .sorting_fn_named("custom_text"),
    ];

    let column_by_id: std::collections::HashMap<&str, &ColumnDef<FixtureRow>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();
        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        if !snap.actions.is_empty() {
            for action in &snap.actions {
                match action {
                    FixtureAction::ToggleSorting { column_id, multi } => {
                        let Some(column) = column_by_id.get(column_id.as_str()).copied() else {
                            panic!("unknown action column_id: {}", column_id);
                        };
                        let auto_sort_dir_desc = auto_sort_dir_desc_for_column(&data, column);
                        toggle_sorting_tanstack(
                            &mut state.sorting,
                            column,
                            options,
                            *multi,
                            auto_sort_dir_desc,
                        );
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
                    state.sorting, expected_state.sorting,
                    "snapshot {} next_state.sorting mismatch",
                    snap.id
                );
            }
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .sorting_fn_builtin(
                "custom_text",
                fret_ui_headless::table::BuiltInSortingFn::Text,
            )
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
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

        // Our engine's `row_model()` corresponds to TanStack's `getRowModel()` (post-pagination).
        // Also gate the explicit pagination model against the fixture's `paginated` snapshot.
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
