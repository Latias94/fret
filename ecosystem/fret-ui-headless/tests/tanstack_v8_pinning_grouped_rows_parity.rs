use std::collections::BTreeMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    Aggregation, ColumnDef, RowId, RowKey, RowPinPosition, Table, TableState, TanStackTableOptions,
    TanStackTableState,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: u64,
    team: u64,
    score: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
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
        .filter_map(|&i| model.row(i).map(|r| r.id.as_ref().to_string()))
        .collect();
    let flat = model
        .flat_rows()
        .iter()
        .filter_map(|&i| model.row(i).map(|r| r.id.as_ref().to_string()))
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

fn tanstack_state_to_table_state_with_row_models(
    data: &[FixtureRow],
    columns: &[ColumnDef<FixtureRow>],
    options: fret_ui_headless::table::TableOptions,
    initial_state: &TableState,
    state_json: &serde_json::Value,
) -> TableState {
    let mut stripped = state_json.clone();
    if let Some(obj) = stripped.as_object_mut() {
        obj.remove("rowPinning");
        obj.remove("rowSelection");
        obj.remove("expanded");
    }

    let tanstack_stripped = TanStackTableState::from_json(&stripped).expect("tanstack state");
    let state_for_models = tanstack_stripped
        .to_table_state()
        .expect("state conversion (stripped)");

    let model_table = Table::builder(data)
        .columns(columns.to_vec())
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
        .initial_state(initial_state.clone())
        .state(state_for_models.clone())
        .options(options.clone())
        .build();

    let core = model_table.core_row_model();
    let grouped = (!state_for_models.grouping.is_empty()).then(|| model_table.grouped_row_model());

    let tanstack_full = TanStackTableState::from_json(state_json).expect("tanstack state");
    tanstack_full
        .to_table_state_with_row_models(core, grouped)
        .expect("state conversion (row models)")
}

#[test]
fn tanstack_v8_pinning_grouped_rows_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("pinning_grouped_rows.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "pinning_grouped_rows");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("role").facet_key_by(|row: &FixtureRow| row.role),
        ColumnDef::<FixtureRow>::new("team").facet_key_by(|row: &FixtureRow| row.team),
        ColumnDef::<FixtureRow>::new("score")
            .value_u64_by(|row: &FixtureRow| row.score)
            .aggregate(Aggregation::SumU64),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let initial_state = match snap.options.get("initialState") {
            Some(v) => TanStackTableState::from_json(v)
                .expect("tanstack initialState")
                .to_table_state()
                .expect("initialState conversion"),
            None => TableState::default(),
        };

        let mut state = tanstack_state_to_table_state_with_row_models(
            &data,
            &columns,
            options.clone(),
            &initial_state,
            &snap.state,
        );

        for action in &snap.actions {
            let table = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .initial_state(initial_state.clone())
                .state(state.clone())
                .options(options.clone())
                .build();

            match action {
                FixtureAction::PinRow {
                    row_id,
                    position,
                    include_leaf_rows,
                    include_parent_rows,
                } => {
                    let pos = parse_row_pin_position(position.as_deref());
                    let updater = table
                        .row_pinning_updater_by_id(
                            row_id.as_str(),
                            true,
                            pos,
                            *include_leaf_rows,
                            *include_parent_rows,
                        )
                        .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                    state.row_pinning = updater.apply(&state.row_pinning);
                }
            }
        }

        if let Some(expected_next) = snap.expect.next_state.as_ref() {
            let expected_state = tanstack_state_to_table_state_with_row_models(
                &data,
                &columns,
                options.clone(),
                &initial_state,
                expected_next,
            );
            assert_eq!(
                state.row_pinning, expected_state.row_pinning,
                "snapshot {} next_state.rowPinning mismatch",
                snap.id
            );
        }

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
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

        // Today `row_model()` corresponds to TanStack's `getRowModel()` (post-pagination).
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

        if let Some(expected) = snap.expect.row_pinning.as_ref() {
            let top: Vec<String> = table
                .top_row_ids()
                .into_iter()
                .map(|id| id.as_ref().to_string())
                .collect();
            let center: Vec<String> = table
                .center_row_ids()
                .into_iter()
                .map(|id| id.as_ref().to_string())
                .collect();
            let bottom: Vec<String> = table
                .bottom_row_ids()
                .into_iter()
                .map(|id| id.as_ref().to_string())
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
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                let can_pin = table
                    .row_can_pin(row_key)
                    .unwrap_or_else(|| panic!("unknown row key for id: {row_id}"));
                assert_eq!(
                    can_pin, *expected_can_pin,
                    "snapshot {} can_pin[{}] mismatch",
                    snap.id, row_id
                );
            }

            for (row_id, expected_pos) in &expected.pin_position {
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
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
                let row_key = table
                    .row_key_for_id(row_id.as_str(), true)
                    .unwrap_or_else(|| panic!("unknown row id: {row_id}"));
                let index = table
                    .row_pinned_index(row_key)
                    .unwrap_or_else(|| panic!("unknown row key for id: {row_id}"));
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
