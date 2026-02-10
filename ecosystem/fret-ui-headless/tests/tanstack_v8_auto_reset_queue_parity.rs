use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, FilteringFnSpec, RowId, RowKey, Table, TableState, TanStackAutoResetQueue,
    TanStackTableOptions, TanStackTableState, TanStackValue,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    next_state: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    #[serde(default)]
    state: serde_json::Value,
    passes: usize,
    expect: FixtureExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

fn tanstack_value_str(s: &str) -> TanStackValue {
    TanStackValue::String(Arc::<str>::from(s))
}

fn tanstack_value_num(n: u64) -> TanStackValue {
    TanStackValue::Number(n as f64)
}

#[test]
fn tanstack_v8_auto_reset_queue_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("auto_reset_queue.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "auto_reset_queue");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name")
            .sort_value_by(|row: &FixtureRow| tanstack_value_str(&row.name))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("status")
            .sort_value_by(|row: &FixtureRow| tanstack_value_str(&row.status))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("cpu")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.cpu))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_value_by(|row: &FixtureRow| tanstack_value_num(row.mem_mb))
            .sorting_fn_auto()
            .filtering_fn_auto(),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        let mut auto_reset = TanStackAutoResetQueue::default();

        for _pass in 0..snap.passes {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options)
                .global_filter_fn(FilteringFnSpec::Auto)
                .build();

            auto_reset.begin_render_pass();
            auto_reset.auto_reset_page_index(&table);
            auto_reset.auto_reset_page_index(&table);
            auto_reset.auto_reset_page_index(&table);
            auto_reset.auto_reset_expanded(&table);
            auto_reset.auto_reset_expanded(&table);
            auto_reset.auto_reset_expanded(&table);
            auto_reset.flush(&table, &mut state);
        }

        let tanstack_next =
            TanStackTableState::from_json(&snap.expect.next_state).expect("tanstack next_state");
        let expected_state = tanstack_next
            .to_table_state()
            .expect("next_state conversion");

        assert_eq!(
            state.pagination, expected_state.pagination,
            "snapshot {} next_state.pagination mismatch",
            snap.id
        );
        assert_eq!(
            state.expanding, expected_state.expanding,
            "snapshot {} next_state.expanded mismatch",
            snap.id
        );
    }
}
