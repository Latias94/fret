use std::path::PathBuf;

use std::collections::BTreeMap;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, FilteringFnSpec, RowKey, RowPinPosition, Table, TanStackTableOptions,
    TanStackTableState, TanStackValue,
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
struct RowPinningExpect {
    top: Vec<String>,
    center: Vec<String>,
    bottom: Vec<String>,
    #[serde(default)]
    can_pin: BTreeMap<String, bool>,
    #[serde(default)]
    pin_position: BTreeMap<String, Option<String>>,
    is_some_rows_pinned: bool,
    is_some_top_rows_pinned: bool,
    is_some_bottom_rows_pinned: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    #[serde(default)]
    row_pinning: Option<RowPinningExpect>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "pinRow")]
    PinRow {
        row_id: String,
        position: Option<String>,
        #[serde(default)]
        include_leaf_rows: bool,
        #[serde(default)]
        include_parent_rows: bool,
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

fn parse_row_pin_position(position: Option<&str>) -> Option<RowPinPosition> {
    match position {
        None => None,
        Some("top") => Some(RowPinPosition::Top),
        Some("bottom") => Some(RowPinPosition::Bottom),
        Some(other) => panic!("invalid pin position: {other}"),
    }
}

fn tanstack_value_str(s: &str) -> TanStackValue {
    TanStackValue::String(Arc::<str>::from(s))
}

fn tanstack_value_num(n: u64) -> TanStackValue {
    TanStackValue::Number(n as f64)
}

#[test]
fn tanstack_v8_pinning_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("pinning.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "pinning");

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

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .global_filter_fn(FilteringFnSpec::Auto)
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::PinRow {
                    row_id,
                    position,
                    include_leaf_rows,
                    include_parent_rows,
                } => {
                    let row_key = RowKey(
                        row_id
                            .parse::<u64>()
                            .unwrap_or_else(|_| panic!("invalid row_id: {row_id}")),
                    );
                    let pos = parse_row_pin_position(position.as_deref());
                    let updater = table.row_pinning_updater(
                        row_key,
                        pos,
                        *include_leaf_rows,
                        *include_parent_rows,
                    );
                    state.row_pinning = updater.apply(&state.row_pinning);
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
                state.row_pinning, expected_state.row_pinning,
                "snapshot {} next_state.rowPinning mismatch",
                snap.id
            );
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .global_filter_fn(FilteringFnSpec::Auto)
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .state(state)
            .options(options)
            .build();

        if let Some(expected) = snap.expect.row_pinning.as_ref() {
            let top: Vec<String> = table
                .top_row_keys()
                .into_iter()
                .map(|k| k.0.to_string())
                .collect();
            let center: Vec<String> = table
                .center_row_keys()
                .into_iter()
                .map(|k| k.0.to_string())
                .collect();
            let bottom: Vec<String> = table
                .bottom_row_keys()
                .into_iter()
                .map(|k| k.0.to_string())
                .collect();

            assert_eq!(
                top, expected.top,
                "snapshot {} row_pinning.top mismatch",
                snap.id
            );
            assert_eq!(
                center, expected.center,
                "snapshot {} row_pinning.center mismatch",
                snap.id
            );
            assert_eq!(
                bottom, expected.bottom,
                "snapshot {} row_pinning.bottom mismatch",
                snap.id
            );

            for (row_id, expected_can_pin) in &expected.can_pin {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                let can_pin = table
                    .row_can_pin(row_key)
                    .unwrap_or_else(|| panic!("unknown row: {row_id}"));
                assert_eq!(
                    can_pin, *expected_can_pin,
                    "snapshot {} can_pin[{}] mismatch",
                    snap.id, row_id
                );
            }

            for (row_id, expected_pos) in &expected.pin_position {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                let pos = table.row_is_pinned(row_key).map(|p| match p {
                    RowPinPosition::Top => "top",
                    RowPinPosition::Bottom => "bottom",
                });
                assert_eq!(
                    pos.as_deref(),
                    expected_pos.as_deref(),
                    "snapshot {} pin_position[{}] mismatch",
                    snap.id,
                    row_id
                );
            }

            assert_eq!(
                table.is_some_rows_pinned(None),
                expected.is_some_rows_pinned,
                "snapshot {} is_some_rows_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_rows_pinned(Some(RowPinPosition::Top)),
                expected.is_some_top_rows_pinned,
                "snapshot {} is_some_top_rows_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_rows_pinned(Some(RowPinPosition::Bottom)),
                expected.is_some_bottom_rows_pinned,
                "snapshot {} is_some_bottom_rows_pinned mismatch",
                snap.id
            );
        }
    }
}
