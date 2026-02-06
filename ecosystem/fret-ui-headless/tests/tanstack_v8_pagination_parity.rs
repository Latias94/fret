use std::path::PathBuf;

use fret_ui_headless::table::{
    ColumnDef, RowId, RowKey, Table, TanStackTableOptions, TanStackTableState,
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
struct RowModelSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
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
    #[serde(default)]
    next_state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum FixtureAction {
    #[serde(rename = "setPageIndex")]
    SetPageIndex { page_index: i32 },
    #[serde(rename = "setPageSize")]
    SetPageSize { page_size: i32 },
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
fn tanstack_v8_pagination_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("pagination.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "pagination");

    let data = fixture.data;

    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("name")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.name.cmp(&b.name)),
        ColumnDef::<FixtureRow>::new("status")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.status.cmp(&b.status)),
        ColumnDef::<FixtureRow>::new("cpu")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu)),
        ColumnDef::<FixtureRow>::new("cpu_desc_first")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .sort_desc_first(true),
        ColumnDef::<FixtureRow>::new("cpu_no_multi")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .enable_multi_sort(false),
        ColumnDef::<FixtureRow>::new("cpu_no_sort")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .enable_sorting(false),
        ColumnDef::<FixtureRow>::new("cpu_invert")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.cpu.cmp(&b.cpu))
            .invert_sorting(true),
        ColumnDef::<FixtureRow>::new("mem_mb")
            .sort_by(|a: &FixtureRow, b: &FixtureRow| a.mem_mb.cmp(&b.mem_mb)),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let mut state = tanstack_state.to_table_state().expect("state conversion");

        let on_pagination_change_mode = snap
            .options
            .get("__onPaginationChange")
            .and_then(|v| v.as_str());
        let override_pagination_model = snap
            .options
            .get("__getPaginationRowModel")
            .and_then(|v| v.as_str())
            == Some("pre_pagination");

        for action in &snap.actions {
            let mut builder = Table::builder(&data)
                .columns(columns.clone())
                .get_row_key(|row, _idx, _parent| RowKey(row.id))
                .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
                .state(state.clone())
                .options(options);

            if override_pagination_model {
                builder = builder.override_pagination_row_model_pre_pagination();
            }

            let table = builder.build();

            if on_pagination_change_mode == Some("noop") {
                continue;
            }

            match action {
                FixtureAction::SetPageIndex { page_index } => {
                    let updater = table.pagination_updater_set_page_index(*page_index);
                    state.pagination = updater.apply(&state.pagination);
                }
                FixtureAction::SetPageSize { page_size } => {
                    let updater = table.pagination_updater_set_page_size(*page_size);
                    state.pagination = updater.apply(&state.pagination);
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
                state.pagination, expected_state.pagination,
                "snapshot {} next_state.pagination mismatch",
                snap.id
            );
        }

        let mut builder = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options);

        if override_pagination_model {
            builder = builder.override_pagination_row_model_pre_pagination();
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
    }
}
