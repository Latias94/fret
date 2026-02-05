use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, ColumnPinPosition, RowKey, Table, TanStackTableOptions, TanStackTableState,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    cpu: u64,
    #[allow(dead_code)]
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnPinningExpect {
    left: Vec<String>,
    center: Vec<String>,
    right: Vec<String>,
    can_pin: BTreeMap<String, bool>,
    pin_position: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pinned_index: BTreeMap<String, i32>,
    is_some_columns_pinned: bool,
    is_some_left_columns_pinned: bool,
    is_some_right_columns_pinned: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    #[serde(default)]
    column_pinning: Option<ColumnPinningExpect>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "pinColumn")]
    PinColumn {
        column_id: String,
        position: Option<String>,
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

fn parse_column_pin_position(position: Option<&str>) -> Option<ColumnPinPosition> {
    match position {
        None => None,
        Some("left") => Some(ColumnPinPosition::Left),
        Some("right") => Some(ColumnPinPosition::Right),
        Some(other) => panic!("invalid pin position: {other}"),
    }
}

fn parse_expected_pin_position(position: Option<&str>) -> Option<ColumnPinPosition> {
    match position {
        None => None,
        Some("left") => Some(ColumnPinPosition::Left),
        Some("right") => Some(ColumnPinPosition::Right),
        Some(other) => panic!("invalid expected pin position: {other}"),
    }
}

#[test]
fn tanstack_v8_column_pinning_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("column_pinning.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "column_pinning");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("a").enable_pinning(true),
        ColumnDef::<FixtureRow>::new("b").enable_pinning(false),
        ColumnDef::<FixtureRow>::new("c").enable_pinning(true),
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
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::PinColumn {
                    column_id,
                    position,
                } => {
                    let pos = parse_column_pin_position(position.as_deref());
                    let updater = table
                        .column_pinning_updater(column_id, pos)
                        .unwrap_or_else(|| panic!("unknown column in action: {column_id}"));
                    state.column_pinning = updater.apply(&state.column_pinning);
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
                state.column_pinning, expected_state.column_pinning,
                "snapshot {} next_state.columnPinning mismatch",
                snap.id
            );
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .state(state)
            .options(options)
            .build();

        if let Some(expected) = snap.expect.column_pinning.as_ref() {
            let (left, center, right) = table.pinned_visible_columns();

            let left: Vec<String> = left
                .into_iter()
                .map(|c| c.id.as_ref().to_string())
                .collect();
            let center: Vec<String> = center
                .into_iter()
                .map(|c| c.id.as_ref().to_string())
                .collect();
            let right: Vec<String> = right
                .into_iter()
                .map(|c| c.id.as_ref().to_string())
                .collect();

            assert_eq!(
                left, expected.left,
                "snapshot {} column_pinning.left mismatch",
                snap.id
            );
            assert_eq!(
                center, expected.center,
                "snapshot {} column_pinning.center mismatch",
                snap.id
            );
            assert_eq!(
                right, expected.right,
                "snapshot {} column_pinning.right mismatch",
                snap.id
            );

            for (col_id, expected_can_pin) in &expected.can_pin {
                let can_pin = table
                    .column_can_pin(col_id)
                    .unwrap_or_else(|| panic!("unknown column: {col_id}"));
                assert_eq!(
                    can_pin, *expected_can_pin,
                    "snapshot {} can_pin[{}] mismatch",
                    snap.id, col_id
                );
            }

            for (col_id, expected_pos) in &expected.pin_position {
                let expected_pos = parse_expected_pin_position(expected_pos.as_deref());
                let pos = table.column_pin_position(col_id);
                assert_eq!(
                    pos, expected_pos,
                    "snapshot {} pin_position[{}] mismatch",
                    snap.id, col_id
                );
            }

            for (col_id, expected_index) in &expected.pinned_index {
                let index = table
                    .column_pinned_index(col_id)
                    .unwrap_or_else(|| panic!("unknown column: {col_id}"));
                assert_eq!(
                    index, *expected_index,
                    "snapshot {} pinned_index[{}] mismatch",
                    snap.id, col_id
                );
            }

            assert_eq!(
                table.is_some_columns_pinned(None),
                expected.is_some_columns_pinned,
                "snapshot {} is_some_columns_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_columns_pinned(Some(ColumnPinPosition::Left)),
                expected.is_some_left_columns_pinned,
                "snapshot {} is_some_left_columns_pinned mismatch",
                snap.id
            );
            assert_eq!(
                table.is_some_columns_pinned(Some(ColumnPinPosition::Right)),
                expected.is_some_right_columns_pinned,
                "snapshot {} is_some_right_columns_pinned mismatch",
                snap.id
            );
        }
    }
}
