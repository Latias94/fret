use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, ColumnPinPosition, ColumnSizingRegion, RowKey, Table, TableState,
    TanStackTableOptions, TanStackTableState,
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
struct FixtureColumnMeta {
    id: String,
    size: f32,
    #[serde(default, rename = "minSize")]
    min_size: Option<f32>,
    #[serde(default, rename = "maxSize")]
    max_size: Option<f32>,
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
    core: RowModelSnapshot,
    filtered: RowModelSnapshot,
    sorted: RowModelSnapshot,
    expanded: RowModelSnapshot,
    paginated: RowModelSnapshot,
    row_model: RowModelSnapshot,
    page_count: i32,
    row_count: usize,
    can_previous_page: bool,
    can_next_page: bool,
    page_options: Vec<usize>,
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
    #[serde(rename = "resetColumnPinning")]
    ResetColumnPinning {
        #[serde(default)]
        default_state: Option<bool>,
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
    columns_meta: Vec<FixtureColumnMeta>,
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

    let column_meta_by_id: HashMap<&str, &FixtureColumnMeta> = fixture
        .columns_meta
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect();

    let col = |id: &str| {
        let meta = column_meta_by_id
            .get(id)
            .unwrap_or_else(|| panic!("missing columns_meta for id: {id}"));

        let mut def = ColumnDef::<FixtureRow>::new(id)
            .size(meta.size)
            .min_size(meta.min_size.unwrap_or(20.0))
            .max_size(meta.max_size.unwrap_or(f32::MAX));

        // Gate per-column `enablePinning` vs table-level options.
        match id {
            "a" | "c" => def = def.enable_pinning(true),
            "b" => def = def.enable_pinning(false),
            _ => {}
        }

        def
    };

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("ab").columns(vec![col("a"), col("b")]),
        col("c"),
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

        let column_pinning_hook_noop = snap
            .options
            .get("__onColumnPinningChange")
            .and_then(|v| v.as_str())
            .is_some_and(|v| v == "noop");

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options)
                .build();

            match action {
                FixtureAction::PinColumn {
                    column_id,
                    position,
                } => {
                    if column_pinning_hook_noop {
                        continue;
                    }
                    let pos = parse_column_pin_position(position.as_deref());
                    let updater = table
                        .column_pinning_updater(column_id, pos)
                        .unwrap_or_else(|| panic!("unknown column in action: {column_id}"));
                    state.column_pinning = updater.apply(&state.column_pinning);
                }
                FixtureAction::ResetColumnPinning { default_state } => {
                    if column_pinning_hook_noop {
                        continue;
                    }
                    state.column_pinning =
                        table.reset_column_pinning(default_state.unwrap_or(false));
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
            .initial_state(initial_state)
            .state(state)
            .options(options)
            .build();

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

        assert_eq!(
            table.page_count(),
            snap.expect.page_count,
            "snapshot {} page_count mismatch",
            snap.id
        );
        assert_eq!(
            table.row_count(),
            snap.expect.row_count,
            "snapshot {} row_count mismatch",
            snap.id
        );
        assert_eq!(
            table.can_previous_page(),
            snap.expect.can_previous_page,
            "snapshot {} can_previous_page mismatch",
            snap.id
        );
        assert_eq!(
            table.can_next_page(),
            snap.expect.can_next_page,
            "snapshot {} can_next_page mismatch",
            snap.id
        );
        assert_eq!(
            table.page_options(),
            snap.expect.page_options,
            "snapshot {} page_options mismatch",
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
