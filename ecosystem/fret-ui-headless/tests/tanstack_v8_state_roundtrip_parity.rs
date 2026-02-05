use std::path::PathBuf;

use fret_ui_headless::table::{ColumnId, ExpandingState, RowKey, TableState, TanStackTableState};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    #[serde(default)]
    state: serde_json::Value,
    #[serde(default)]
    expect: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    snapshots: Vec<FixtureSnapshot>,
}

fn assert_f32_eq(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.0001,
        "{context}: expected {expected}, got {actual} (delta={delta})"
    );
}

fn assert_table_state_eq(actual: &TableState, expected: &TableState, context: &str) {
    let sorting_actual: Vec<(&str, bool)> = actual
        .sorting
        .iter()
        .map(|s| (s.column.as_ref(), s.desc))
        .collect();
    let sorting_expected: Vec<(&str, bool)> = expected
        .sorting
        .iter()
        .map(|s| (s.column.as_ref(), s.desc))
        .collect();
    assert_eq!(
        sorting_actual, sorting_expected,
        "{context}: sorting mismatch"
    );

    assert_eq!(
        actual.column_filters.len(),
        expected.column_filters.len(),
        "{context}: column_filters len mismatch"
    );
    for (i, (a, b)) in actual
        .column_filters
        .iter()
        .zip(expected.column_filters.iter())
        .enumerate()
    {
        assert_eq!(
            a.column.as_ref(),
            b.column.as_ref(),
            "{context}: column_filters[{i}].column mismatch"
        );
        assert_eq!(
            a.value, b.value,
            "{context}: column_filters[{i}].value mismatch"
        );
    }

    assert_eq!(
        actual.global_filter.as_ref(),
        expected.global_filter.as_ref(),
        "{context}: global_filter mismatch"
    );

    assert_eq!(
        actual.pagination.page_index, expected.pagination.page_index,
        "{context}: pagination.page_index mismatch"
    );
    assert_eq!(
        actual.pagination.page_size, expected.pagination.page_size,
        "{context}: pagination.page_size mismatch"
    );

    let grouping_actual: Vec<&str> = actual.grouping.iter().map(|s| s.as_ref()).collect();
    let grouping_expected: Vec<&str> = expected.grouping.iter().map(|s| s.as_ref()).collect();
    assert_eq!(
        grouping_actual, grouping_expected,
        "{context}: grouping mismatch"
    );

    match (&actual.expanding, &expected.expanding) {
        (ExpandingState::All, ExpandingState::All) => {}
        (ExpandingState::Keys(a), ExpandingState::Keys(b)) => {
            let mut a_keys: Vec<u64> = a.iter().map(|k| k.0).collect();
            let mut b_keys: Vec<u64> = b.iter().map(|k| k.0).collect();
            a_keys.sort_unstable();
            b_keys.sort_unstable();
            assert_eq!(a_keys, b_keys, "{context}: expanding keys mismatch");
        }
        _ => panic!("{context}: expanding kind mismatch"),
    }

    let rowpin_top_a: Vec<u64> = actual.row_pinning.top.iter().map(|k| k.0).collect();
    let rowpin_top_b: Vec<u64> = expected.row_pinning.top.iter().map(|k| k.0).collect();
    assert_eq!(
        rowpin_top_a, rowpin_top_b,
        "{context}: row_pinning.top mismatch"
    );
    let rowpin_bottom_a: Vec<u64> = actual.row_pinning.bottom.iter().map(|k| k.0).collect();
    let rowpin_bottom_b: Vec<u64> = expected.row_pinning.bottom.iter().map(|k| k.0).collect();
    assert_eq!(
        rowpin_bottom_a, rowpin_bottom_b,
        "{context}: row_pinning.bottom mismatch"
    );

    let mut sel_a: Vec<u64> = actual.row_selection.iter().map(|k| k.0).collect();
    let mut sel_b: Vec<u64> = expected.row_selection.iter().map(|k| k.0).collect();
    sel_a.sort_unstable();
    sel_b.sort_unstable();
    assert_eq!(sel_a, sel_b, "{context}: row_selection mismatch");

    assert_eq!(
        actual.column_visibility, expected.column_visibility,
        "{context}: column_visibility mismatch"
    );

    let order_a: Vec<&str> = actual.column_order.iter().map(|s| s.as_ref()).collect();
    let order_b: Vec<&str> = expected.column_order.iter().map(|s| s.as_ref()).collect();
    assert_eq!(order_a, order_b, "{context}: column_order mismatch");

    let pin_left_a: Vec<&str> = actual
        .column_pinning
        .left
        .iter()
        .map(|s| s.as_ref())
        .collect();
    let pin_left_b: Vec<&str> = expected
        .column_pinning
        .left
        .iter()
        .map(|s| s.as_ref())
        .collect();
    assert_eq!(
        pin_left_a, pin_left_b,
        "{context}: column_pinning.left mismatch"
    );

    let pin_right_a: Vec<&str> = actual
        .column_pinning
        .right
        .iter()
        .map(|s| s.as_ref())
        .collect();
    let pin_right_b: Vec<&str> = expected
        .column_pinning
        .right
        .iter()
        .map(|s| s.as_ref())
        .collect();
    assert_eq!(
        pin_right_a, pin_right_b,
        "{context}: column_pinning.right mismatch"
    );

    assert_eq!(
        actual.column_sizing.len(),
        expected.column_sizing.len(),
        "{context}: column_sizing len mismatch"
    );
    for (k, v) in &expected.column_sizing {
        let got = actual.column_sizing.get(k).copied().unwrap_or(0.0);
        assert_f32_eq(
            got,
            *v,
            &format!("{context}: column_sizing[{}]", k.as_ref()),
        );
    }

    assert_f32_eq(
        actual.column_sizing_info.start_offset.unwrap_or(-1.0),
        expected.column_sizing_info.start_offset.unwrap_or(-1.0),
        &format!("{context}: column_sizing_info.start_offset"),
    );
    assert_f32_eq(
        actual.column_sizing_info.start_size.unwrap_or(-1.0),
        expected.column_sizing_info.start_size.unwrap_or(-1.0),
        &format!("{context}: column_sizing_info.start_size"),
    );
    assert_f32_eq(
        actual.column_sizing_info.delta_offset.unwrap_or(-1.0),
        expected.column_sizing_info.delta_offset.unwrap_or(-1.0),
        &format!("{context}: column_sizing_info.delta_offset"),
    );
    assert_f32_eq(
        actual.column_sizing_info.delta_percentage.unwrap_or(-1.0),
        expected.column_sizing_info.delta_percentage.unwrap_or(-1.0),
        &format!("{context}: column_sizing_info.delta_percentage"),
    );
    assert_eq!(
        actual
            .column_sizing_info
            .is_resizing_column
            .as_ref()
            .map(|s| s.as_ref()),
        expected
            .column_sizing_info
            .is_resizing_column
            .as_ref()
            .map(|s| s.as_ref()),
        "{context}: column_sizing_info.is_resizing_column mismatch"
    );
    assert_eq!(
        actual.column_sizing_info.column_sizing_start.len(),
        expected.column_sizing_info.column_sizing_start.len(),
        "{context}: column_sizing_info.column_sizing_start len mismatch"
    );
    for (i, (id, size)) in expected
        .column_sizing_info
        .column_sizing_start
        .iter()
        .enumerate()
    {
        let (a_id, a_size) = &actual.column_sizing_info.column_sizing_start[i];
        assert_eq!(
            a_id.as_ref(),
            id.as_ref(),
            "{context}: column_sizing_info.column_sizing_start[{i}].id mismatch"
        );
        assert_f32_eq(
            *a_size,
            *size,
            &format!("{context}: column_sizing_info.column_sizing_start[{i}].size"),
        );
    }
}

fn load_fixture(path: &PathBuf) -> Fixture {
    serde_json::from_str(&std::fs::read_to_string(path).expect("fixture file")).expect("fixture")
}

fn state_roundtrip(state_json: &serde_json::Value) -> TableState {
    let tanstack = TanStackTableState::from_json(state_json).expect("tanstack state");
    let state = tanstack.to_table_state().expect("to_table_state");
    let json2 = TanStackTableState::from_table_state(&state)
        .to_json()
        .expect("to_json");
    let tanstack2 = TanStackTableState::from_json(&json2).expect("tanstack state 2");
    tanstack2.to_table_state().expect("to_table_state 2")
}

#[test]
fn tanstack_v8_state_roundtrip_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8");

    // We only assert round-trips for the subset we currently model in `TanStackTableState`.
    for file in [
        "demo_process.json",
        "sort_undefined.json",
        "sorting_fns.json",
        "filtering_fns.json",
        "column_sizing.json",
        "headers_cells.json",
        "state_shapes.json",
        "selection.json",
        "expanding.json",
    ] {
        let fixture = load_fixture(&base.join(file));
        for snap in fixture.snapshots {
            let expected = TanStackTableState::from_json(&snap.state)
                .expect("tanstack state")
                .to_table_state()
                .expect("to_table_state");
            let actual = state_roundtrip(&snap.state);
            assert_table_state_eq(
                &actual,
                &expected,
                &format!("fixture {} snapshot {}", fixture.case_id, snap.id),
            );

            // If the fixture has a `next_state`, round-trip it too (column sizing actions rely on this).
            if let Some(next_state) = snap.expect.get("next_state") {
                let expected_next = TanStackTableState::from_json(next_state)
                    .expect("tanstack next_state")
                    .to_table_state()
                    .expect("to_table_state next");
                let actual_next = state_roundtrip(next_state);
                assert_table_state_eq(
                    &actual_next,
                    &expected_next,
                    &format!(
                        "fixture {} snapshot {} next_state",
                        fixture.case_id, snap.id
                    ),
                );
            }
        }
    }
}

// Touch types so rustc doesn't optimize away imports in case this test evolves.
#[allow(dead_code)]
fn _touch_types() {
    let _ = ColumnId::from("x");
    let _ = RowKey(1);
}
