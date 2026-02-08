use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, ColumnId, ColumnSizingRegion, RowId, RowKey, Table, TableState,
    TanStackTableOptions, TanStackTableState, begin_column_resize, drag_column_resize,
    end_column_resize, resolved_column_size,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureColumnMeta {
    id: String,
    size: f32,
    #[serde(default, rename = "minSize")]
    min_size: Option<f32>,
    #[serde(default, rename = "maxSize")]
    max_size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnSizingExpect {
    total_size: f32,
    left_total_size: f32,
    center_total_size: f32,
    right_total_size: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnStartExpect {
    all: HashMap<String, f32>,
    left: HashMap<String, Option<f32>>,
    center: HashMap<String, Option<f32>>,
    right: HashMap<String, Option<f32>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ColumnAfterExpect {
    all: HashMap<String, f32>,
    left: HashMap<String, Option<f32>>,
    center: HashMap<String, Option<f32>>,
    right: HashMap<String, Option<f32>>,
}

#[derive(Debug, Clone, Deserialize)]
struct HeaderSizingExpect {
    size: HashMap<String, f32>,
    start: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    #[serde(rename = "column_sizing")]
    sizing: ColumnSizingExpect,
    #[serde(rename = "column_start")]
    starts: ColumnStartExpect,
    #[serde(rename = "column_after")]
    after: ColumnAfterExpect,
    #[serde(default, rename = "header_sizing")]
    header_sizing: Option<HeaderSizingExpect>,
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "columnResizeBegin")]
    ColumnResizeBegin { column_id: String, client_x: f32 },
    #[serde(rename = "columnResizeMove")]
    ColumnResizeMove { client_x: f32 },
    #[serde(rename = "columnResizeEnd")]
    ColumnResizeEnd { client_x: f32 },
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
    columns_meta: Vec<FixtureColumnMeta>,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct TanStackColumnPinningState {
    #[serde(default)]
    left: Vec<String>,
    #[serde(default)]
    right: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TanStackStateSubset {
    #[serde(default, rename = "columnOrder")]
    column_order: Vec<String>,
    #[serde(default, rename = "columnPinning")]
    column_pinning: Option<TanStackColumnPinningState>,
    #[serde(default, rename = "columnSizing")]
    column_sizing: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct TanStackColumnSizingInfoExpected {
    #[serde(default, rename = "columnSizingStart")]
    column_sizing_start: Vec<(String, f32)>,
    #[serde(default, rename = "deltaOffset")]
    delta_offset: Option<f32>,
    #[serde(default, rename = "deltaPercentage")]
    delta_percentage: Option<f32>,
    #[serde(default, rename = "isResizingColumn")]
    is_resizing_column: serde_json::Value,
    #[serde(default, rename = "startOffset")]
    start_offset: Option<f32>,
    #[serde(default, rename = "startSize")]
    start_size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
struct TanStackNextStateSubset {
    #[serde(default, rename = "columnSizingInfo")]
    column_sizing_info: Option<TanStackColumnSizingInfoExpected>,
}

fn assert_f32_eq(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.0001,
        "{context}: expected {expected}, got {actual} (delta={delta})"
    );
}

fn leaf_column_ids_under_header(
    table: &Table<'_, FixtureRow>,
    header_id: Arc<str>,
) -> Vec<Arc<str>> {
    let groups = table.header_groups();
    let mut by_id: HashMap<Arc<str>, fret_ui_headless::table::HeaderSnapshot> = HashMap::new();
    for g in groups {
        for h in g.headers {
            by_id.insert(h.id.clone(), h);
        }
    }

    fn collect(
        by_id: &HashMap<Arc<str>, fret_ui_headless::table::HeaderSnapshot>,
        header_id: &Arc<str>,
        out: &mut Vec<Arc<str>>,
    ) {
        let Some(h) = by_id.get(header_id) else {
            return;
        };
        if h.sub_header_ids.is_empty() {
            out.push(h.column_id.clone());
            return;
        }
        for child in &h.sub_header_ids {
            collect(by_id, child, out);
        }
    }

    let mut out = Vec::new();
    collect(&by_id, &header_id, &mut out);
    out
}

#[test]
fn tanstack_v8_column_resizing_group_headers_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("column_resizing_group_headers.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "column_resizing_group_headers");

    let data = fixture.data;

    let mut leaf_by_id: HashMap<&str, ColumnDef<FixtureRow>> = HashMap::new();
    for m in &fixture.columns_meta {
        let mut col = ColumnDef::<FixtureRow>::new(m.id.as_str())
            .size(m.size)
            .sort_by(|_a: &FixtureRow, _b: &FixtureRow| std::cmp::Ordering::Equal);
        if let Some(min) = m.min_size {
            col = col.min_size(min);
        }
        if let Some(max) = m.max_size {
            col = col.max_size(max);
        }
        leaf_by_id.insert(m.id.as_str(), col);
    }

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("ab").columns(vec![
            leaf_by_id.remove("a").expect("leaf a"),
            leaf_by_id.remove("b").expect("leaf b"),
        ]),
        leaf_by_id.remove("c").expect("leaf c"),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let subset: TanStackStateSubset =
            serde_json::from_value(snap.state.clone()).expect("tanstack state subset");

        let mut state = TableState::default();
        state.column_order = subset
            .column_order
            .iter()
            .map(|s| Arc::<str>::from(s.as_str()))
            .collect();
        if let Some(pin) = subset.column_pinning {
            state.column_pinning.left = pin
                .left
                .iter()
                .map(|s| Arc::<str>::from(s.as_str()))
                .collect();
            state.column_pinning.right = pin
                .right
                .iter()
                .map(|s| Arc::<str>::from(s.as_str()))
                .collect();
        }
        for (k, v) in subset.column_sizing {
            state.column_sizing.insert(Arc::<str>::from(k.as_str()), v);
        }

        let mut active_resize: Option<ColumnId> = None;
        for action in &snap.actions {
            match action {
                FixtureAction::ColumnResizeBegin {
                    column_id,
                    client_x,
                } => {
                    let table = Table::builder(&data)
                        .columns(columns.clone())
                        .get_row_key(|row, _idx, _parent| RowKey(row.id))
                        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                        .state(state.clone())
                        .options(options)
                        .build();
                    let groups = table.header_groups();
                    let header = groups
                        .iter()
                        .flat_map(|g| g.headers.iter())
                        .find(|h| !h.is_placeholder && h.column_id.as_ref() == column_id.as_str())
                        .expect("expected a resizeable header for column_id");

                    let leaf_ids = leaf_column_ids_under_header(&table, header.id.clone());
                    assert!(
                        !leaf_ids.is_empty(),
                        "expected header {} to have leaf columns",
                        column_id
                    );

                    let mut start = Vec::with_capacity(leaf_ids.len());
                    let mut start_size = 0.0;
                    for leaf_id in leaf_ids {
                        let col_def = table
                            .column(leaf_id.as_ref())
                            .expect("leaf column def exists");
                        let size = resolved_column_size(&state.column_sizing, col_def);
                        start_size += size;
                        start.push((col_def.id.clone(), size));
                    }
                    // TanStack includes the resizing header itself (including group headers) in
                    // `columnSizingStart`, with `startSize` equal to the sum of its leaf headers.
                    start.push((Arc::<str>::from(column_id.as_str()), start_size));

                    begin_column_resize(
                        &mut state.column_sizing_info,
                        Arc::<str>::from(column_id.as_str()),
                        *client_x,
                        start,
                    );
                    active_resize = Some(Arc::<str>::from(column_id.as_str()));
                }
                FixtureAction::ColumnResizeMove { client_x } => {
                    assert!(
                        active_resize.is_some(),
                        "snapshot {} columnResizeMove without begin",
                        snap.id
                    );
                    drag_column_resize(
                        options.column_resize_mode,
                        options.column_resize_direction,
                        &mut state.column_sizing,
                        &mut state.column_sizing_info,
                        *client_x,
                    );
                }
                FixtureAction::ColumnResizeEnd { client_x } => {
                    assert!(
                        active_resize.is_some(),
                        "snapshot {} columnResizeEnd without begin",
                        snap.id
                    );
                    end_column_resize(
                        options.column_resize_mode,
                        options.column_resize_direction,
                        &mut state.column_sizing,
                        &mut state.column_sizing_info,
                        Some(*client_x),
                    );
                    active_resize = None;
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
                state.column_order, expected_state.column_order,
                "snapshot {} next_state.columnOrder mismatch",
                snap.id
            );
            assert_eq!(
                state.column_pinning.left, expected_state.column_pinning.left,
                "snapshot {} next_state.columnPinning.left mismatch",
                snap.id
            );
            assert_eq!(
                state.column_pinning.right, expected_state.column_pinning.right,
                "snapshot {} next_state.columnPinning.right mismatch",
                snap.id
            );

            assert_eq!(
                state.column_sizing.len(),
                expected_state.column_sizing.len(),
                "snapshot {} next_state.columnSizing len mismatch",
                snap.id
            );
            for (k, expected_v) in &expected_state.column_sizing {
                let actual_v = state.column_sizing.get(k).copied().unwrap_or(0.0);
                assert_f32_eq(
                    actual_v,
                    *expected_v,
                    &format!(
                        "snapshot {} next_state.columnSizing[{}]",
                        snap.id,
                        k.as_ref()
                    ),
                );
            }

            let expected: TanStackNextStateSubset =
                serde_json::from_value(expected_next.clone()).expect("next_state subset");
            if let Some(info) = expected.column_sizing_info.as_ref() {
                assert_f32_eq(
                    state.column_sizing_info.start_offset.unwrap_or(-1.0),
                    info.start_offset.unwrap_or(-1.0),
                    &format!(
                        "snapshot {} next_state.columnSizingInfo.startOffset",
                        snap.id
                    ),
                );
                assert_f32_eq(
                    state.column_sizing_info.start_size.unwrap_or(-1.0),
                    info.start_size.unwrap_or(-1.0),
                    &format!("snapshot {} next_state.columnSizingInfo.startSize", snap.id),
                );
                assert_f32_eq(
                    state.column_sizing_info.delta_offset.unwrap_or(-1.0),
                    info.delta_offset.unwrap_or(-1.0),
                    &format!(
                        "snapshot {} next_state.columnSizingInfo.deltaOffset",
                        snap.id
                    ),
                );
                assert_f32_eq(
                    state.column_sizing_info.delta_percentage.unwrap_or(-1.0),
                    info.delta_percentage.unwrap_or(-1.0),
                    &format!(
                        "snapshot {} next_state.columnSizingInfo.deltaPercentage",
                        snap.id
                    ),
                );

                let expected_resizing = match &info.is_resizing_column {
                    serde_json::Value::String(s) => Some(s.as_str()),
                    serde_json::Value::Bool(false) => None,
                    serde_json::Value::Null => None,
                    v => panic!(
                        "snapshot {} next_state.columnSizingInfo.isResizingColumn invalid: {v:?}",
                        snap.id
                    ),
                };
                let actual_resizing = state
                    .column_sizing_info
                    .is_resizing_column
                    .as_ref()
                    .map(|s| s.as_ref());
                assert_eq!(
                    actual_resizing, expected_resizing,
                    "snapshot {} next_state.columnSizingInfo.isResizingColumn mismatch",
                    snap.id
                );

                assert_eq!(
                    state.column_sizing_info.column_sizing_start.len(),
                    info.column_sizing_start.len(),
                    "snapshot {} next_state.columnSizingInfo.columnSizingStart len mismatch",
                    snap.id
                );
                for (i, (id, size)) in info.column_sizing_start.iter().enumerate() {
                    let (actual_id, actual_size) = &state.column_sizing_info.column_sizing_start[i];
                    assert_eq!(
                        actual_id.as_ref(),
                        id.as_str(),
                        "snapshot {} next_state.columnSizingInfo.columnSizingStart[{i}].id mismatch",
                        snap.id
                    );
                    assert_f32_eq(
                        *actual_size,
                        *size,
                        &format!(
                            "snapshot {} next_state.columnSizingInfo.columnSizingStart[{i}].size",
                            snap.id
                        ),
                    );
                }
            }
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .build();

        assert_f32_eq(
            table.total_size(),
            snap.expect.sizing.total_size,
            &format!("snapshot {} total_size", snap.id),
        );
        assert_f32_eq(
            table.left_total_size(),
            snap.expect.sizing.left_total_size,
            &format!("snapshot {} left_total_size", snap.id),
        );
        assert_f32_eq(
            table.center_total_size(),
            snap.expect.sizing.center_total_size,
            &format!("snapshot {} center_total_size", snap.id),
        );
        assert_f32_eq(
            table.right_total_size(),
            snap.expect.sizing.right_total_size,
            &format!("snapshot {} right_total_size", snap.id),
        );

        for (col_id, expected) in &snap.expect.starts.all {
            let actual = table
                .column_start(col_id.as_str(), ColumnSizingRegion::All)
                .unwrap_or(-1.0);
            assert_f32_eq(
                actual,
                *expected,
                &format!("snapshot {} column_start(all,{col_id})", snap.id),
            );
        }

        for (col_id, expected) in &snap.expect.after.all {
            let actual = table
                .column_after(col_id.as_str(), ColumnSizingRegion::All)
                .unwrap_or(-1.0);
            assert_f32_eq(
                actual,
                *expected,
                &format!("snapshot {} column_after(all,{col_id})", snap.id),
            );
        }

        for (col_id, expected) in &snap.expect.starts.left {
            let actual = table.column_start(col_id.as_str(), ColumnSizingRegion::Left);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_start(left,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_start(left,{col_id})", snap.id),
                );
            }
        }

        for (col_id, expected) in &snap.expect.after.left {
            let actual = table.column_after(col_id.as_str(), ColumnSizingRegion::Left);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_after(left,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_after(left,{col_id})", snap.id),
                );
            }
        }

        for (col_id, expected) in &snap.expect.starts.center {
            let actual = table.column_start(col_id.as_str(), ColumnSizingRegion::Center);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_start(center,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_start(center,{col_id})", snap.id),
                );
            }
        }

        for (col_id, expected) in &snap.expect.after.center {
            let actual = table.column_after(col_id.as_str(), ColumnSizingRegion::Center);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_after(center,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_after(center,{col_id})", snap.id),
                );
            }
        }

        for (col_id, expected) in &snap.expect.starts.right {
            let actual = table.column_start(col_id.as_str(), ColumnSizingRegion::Right);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_start(right,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_start(right,{col_id})", snap.id),
                );
            }
        }

        for (col_id, expected) in &snap.expect.after.right {
            let actual = table.column_after(col_id.as_str(), ColumnSizingRegion::Right);
            assert_eq!(
                actual.is_some(),
                expected.is_some(),
                "snapshot {} column_after(right,{col_id}) presence mismatch",
                snap.id
            );
            if let (Some(actual), Some(expected)) = (actual, expected) {
                assert_f32_eq(
                    actual,
                    *expected,
                    &format!("snapshot {} column_after(right,{col_id})", snap.id),
                );
            }
        }

        if let Some(expected) = snap.expect.header_sizing.as_ref() {
            for (header_id, expected_size) in &expected.size {
                let actual = table.header_size(header_id.as_str()).unwrap_or(-1.0);
                assert_f32_eq(
                    actual,
                    *expected_size,
                    &format!("snapshot {} header_size({header_id})", snap.id),
                );
            }
            for (header_id, expected_start) in &expected.start {
                let actual = table.header_start(header_id.as_str()).unwrap_or(-1.0);
                assert_f32_eq(
                    actual,
                    *expected_start,
                    &format!("snapshot {} header_start({header_id})", snap.id),
                );
            }
        }
    }
}
