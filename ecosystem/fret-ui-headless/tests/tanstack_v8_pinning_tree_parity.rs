use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, ColumnSizingRegion, FilteringFnSpec, RowId, RowKey, RowPinPosition, Table,
    TableState, TanStackTableOptions, TanStackTableState, TanStackValue,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    name: String,
    status: String,
    cpu: u64,
    mem_mb: u64,
    #[serde(default, rename = "subRows")]
    sub_rows: Vec<FixtureRow>,
}

#[derive(Debug, Clone, Deserialize)]
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
struct RowPinningExpect {
    top: Vec<String>,
    center: Vec<String>,
    bottom: Vec<String>,
    #[serde(default)]
    can_pin: BTreeMap<String, bool>,
    #[serde(default)]
    pin_position: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pinned_index: BTreeMap<String, i32>,
    is_some_rows_pinned: bool,
    is_some_top_rows_pinned: bool,
    is_some_bottom_rows_pinned: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    expanded: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    selected: RowModelSnapshot,
    filtered_selected: RowModelSnapshot,
    grouped_selected: RowModelSnapshot,
    is_all_rows_selected: bool,
    is_some_rows_selected: bool,
    is_all_page_rows_selected: bool,
    is_some_page_rows_selected: bool,
    is_all_rows_expanded: bool,
    is_some_rows_expanded: bool,
    can_some_rows_expand: bool,
    #[serde(rename = "column_sizing")]
    #[serde(default)]
    sizing: Option<ColumnSizingExpect>,
    #[serde(rename = "column_start")]
    #[serde(default)]
    starts: Option<ColumnStartExpect>,
    #[serde(rename = "column_after")]
    #[serde(default)]
    after: Option<ColumnAfterExpect>,
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
fn tanstack_v8_pinning_tree_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("pinning_tree.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "pinning_tree");

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

        // TanStack: table-level reset APIs target `options.initialState`, not `options.state`.
        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        let enable_row_pinning_mode = snap
            .options
            .get("__enableRowPinning")
            .and_then(|v| v.as_str());

        for action in &snap.actions {
            let mut builder = Table::builder(&data)
                .columns(columns.clone())
                .global_filter_fn(FilteringFnSpec::Auto)
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .get_sub_rows(|row, _idx| Some(row.sub_rows.as_slice()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options);

            if enable_row_pinning_mode == Some("odd_ids") {
                builder = builder.enable_row_pinning_by(|row_key, _row| row_key.0 % 2 == 1);
            }

            let table = builder.build();

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

        let mut builder = Table::builder(&data)
            .columns(columns.clone())
            .global_filter_fn(FilteringFnSpec::Auto)
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .get_sub_rows(|row, _idx| Some(row.sub_rows.as_slice()))
            .initial_state(initial_state)
            .state(state)
            .options(options);

        if enable_row_pinning_mode == Some("odd_ids") {
            builder = builder.enable_row_pinning_by(|row_key, _row| row_key.0 % 2 == 1);
        }

        let table = builder.build();

        let core = snapshot_row_model(table.core_row_model());
        let filtered = snapshot_row_model(table.filtered_row_model());
        let sorted = snapshot_row_model(table.sorted_row_model());
        let expanded = snapshot_row_model(table.expanded_row_model());
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
            expanded.root, snap.expect.expanded.root,
            "snapshot {} expanded root mismatch",
            snap.id
        );
        assert_eq!(
            expanded.flat, snap.expect.expanded.flat,
            "snapshot {} expanded flat mismatch",
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

        // Today `row_model()` corresponds to TanStack's `getRowModel()` (post-pagination) and
        // matches `getPaginationRowModel()` when expansion is inactive.
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

        let selected = snapshot_row_model(table.selected_row_model());
        assert_eq!(
            selected.root, snap.expect.selected.root,
            "snapshot {} selected root mismatch",
            snap.id
        );
        assert_eq!(
            selected.flat, snap.expect.selected.flat,
            "snapshot {} selected flat mismatch",
            snap.id
        );

        let filtered_selected = snapshot_row_model(table.filtered_selected_row_model());
        assert_eq!(
            filtered_selected.root, snap.expect.filtered_selected.root,
            "snapshot {} filtered_selected root mismatch",
            snap.id
        );
        assert_eq!(
            filtered_selected.flat, snap.expect.filtered_selected.flat,
            "snapshot {} filtered_selected flat mismatch",
            snap.id
        );

        let grouped_selected = snapshot_row_model(table.grouped_selected_row_model());
        assert_eq!(
            grouped_selected.root, snap.expect.grouped_selected.root,
            "snapshot {} grouped_selected root mismatch",
            snap.id
        );
        assert_eq!(
            grouped_selected.flat, snap.expect.grouped_selected.flat,
            "snapshot {} grouped_selected flat mismatch",
            snap.id
        );

        assert_eq!(
            table.is_all_rows_selected(),
            snap.expect.is_all_rows_selected,
            "snapshot {} is_all_rows_selected mismatch",
            snap.id
        );
        assert_eq!(
            table.is_some_rows_selected(),
            snap.expect.is_some_rows_selected,
            "snapshot {} is_some_rows_selected mismatch",
            snap.id
        );
        assert_eq!(
            table.is_all_page_rows_selected(),
            snap.expect.is_all_page_rows_selected,
            "snapshot {} is_all_page_rows_selected mismatch",
            snap.id
        );
        assert_eq!(
            table.is_some_page_rows_selected(),
            snap.expect.is_some_page_rows_selected,
            "snapshot {} is_some_page_rows_selected mismatch",
            snap.id
        );

        assert_eq!(
            table.is_all_rows_expanded(),
            snap.expect.is_all_rows_expanded,
            "snapshot {} is_all_rows_expanded mismatch",
            snap.id
        );
        assert_eq!(
            table.is_some_rows_expanded(),
            snap.expect.is_some_rows_expanded,
            "snapshot {} is_some_rows_expanded mismatch",
            snap.id
        );
        assert_eq!(
            table.can_some_rows_expand(),
            snap.expect.can_some_rows_expand,
            "snapshot {} can_some_rows_expand mismatch",
            snap.id
        );

        if let Some(sizing) = snap.expect.sizing.as_ref() {
            assert_eq!(
                table.total_size(),
                sizing.total_size,
                "snapshot {} column_sizing.total_size mismatch",
                snap.id
            );
            assert_eq!(
                table.left_total_size(),
                sizing.left_total_size,
                "snapshot {} column_sizing.left_total_size mismatch",
                snap.id
            );
            assert_eq!(
                table.center_total_size(),
                sizing.center_total_size,
                "snapshot {} column_sizing.center_total_size mismatch",
                snap.id
            );
            assert_eq!(
                table.right_total_size(),
                sizing.right_total_size,
                "snapshot {} column_sizing.right_total_size mismatch",
                snap.id
            );
        }

        if let (Some(starts), Some(after)) =
            (snap.expect.starts.as_ref(), snap.expect.after.as_ref())
        {
            for (col_id, expected) in &starts.all {
                let actual = table.column_start(col_id.as_str(), ColumnSizingRegion::All);
                assert_eq!(
                    actual,
                    Some(*expected),
                    "snapshot {} column_start(all,{col_id}) mismatch",
                    snap.id
                );
            }
            for (col_id, expected) in &after.all {
                let actual = table.column_after(col_id.as_str(), ColumnSizingRegion::All);
                assert_eq!(
                    actual,
                    Some(*expected),
                    "snapshot {} column_after(all,{col_id}) mismatch",
                    snap.id
                );
            }

            let starts = [
                (ColumnSizingRegion::Left, &starts.left, "left"),
                (ColumnSizingRegion::Center, &starts.center, "center"),
                (ColumnSizingRegion::Right, &starts.right, "right"),
            ];
            for (region, by_col, label) in starts {
                for (col_id, expected) in by_col {
                    let actual = table.column_start(col_id.as_str(), region);
                    assert_eq!(
                        actual, *expected,
                        "snapshot {} column_start({label},{col_id}) mismatch",
                        snap.id
                    );
                }
            }

            let after = [
                (ColumnSizingRegion::Left, &after.left, "left"),
                (ColumnSizingRegion::Center, &after.center, "center"),
                (ColumnSizingRegion::Right, &after.right, "right"),
            ];
            for (region, by_col, label) in after {
                for (col_id, expected) in by_col {
                    let actual = table.column_after(col_id.as_str(), region);
                    assert_eq!(
                        actual, *expected,
                        "snapshot {} column_after({label},{col_id}) mismatch",
                        snap.id
                    );
                }
            }
        }

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

            for (row_id, expected_index) in &expected.pinned_index {
                let row_key = RowKey(
                    row_id
                        .parse::<u64>()
                        .unwrap_or_else(|_| panic!("invalid row id: {row_id}")),
                );
                let index = table
                    .row_pinned_index(row_key)
                    .unwrap_or_else(|| panic!("unknown row: {row_id}"));
                assert_eq!(
                    index, *expected_index,
                    "snapshot {} pinned_index[{}] mismatch",
                    snap.id, row_id
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
