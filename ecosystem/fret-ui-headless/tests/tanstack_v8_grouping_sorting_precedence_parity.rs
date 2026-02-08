use std::collections::HashMap;
use std::path::PathBuf;

use fret_ui_headless::table::{
    BuiltInAggregationFn, ColumnDef, GroupedRowIndex, RowId, RowKey, Table, TanStackTableOptions,
    TanStackTableState, TanStackValue, sort_grouped_row_indices_in_place,
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct FixtureRow {
    id: u64,
    role: u64,
    score: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct SortedGroupedIdsSnapshot {
    root: Vec<String>,
    flat: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureExpect {
    sorted_grouped_ids: SortedGroupedIdsSnapshot,
}

#[derive(Debug, Clone, Deserialize)]
struct FixtureSnapshot {
    id: String,
    options: serde_json::Value,
    #[serde(default)]
    state: serde_json::Value,
    expect: FixtureExpect,
}

#[derive(Debug, Clone, Deserialize)]
struct Fixture {
    case_id: String,
    data: Vec<FixtureRow>,
    snapshots: Vec<FixtureSnapshot>,
}

fn row_index_by_key(table: &Table<'_, FixtureRow>) -> HashMap<RowKey, usize> {
    let mut out = HashMap::new();
    let core = table.core_row_model();
    for &row_index in core.flat_rows() {
        let Some(row) = core.row(row_index) else {
            continue;
        };
        out.insert(row.key, row.index);
    }
    out
}

fn snapshot_sorted_grouped_ids(
    table: &Table<'_, FixtureRow>,
    columns: &[ColumnDef<FixtureRow>],
    data: &[FixtureRow],
) -> SortedGroupedIdsSnapshot {
    let model = table.grouped_row_model();
    let mut roots: Vec<GroupedRowIndex> = model.root_rows().to_vec();
    let row_index_by_key = row_index_by_key(table);

    sort_grouped_row_indices_in_place(
        model,
        roots.as_mut_slice(),
        table.state().sorting.as_slice(),
        columns,
        data,
        &row_index_by_key,
        table.grouped_u64_aggregations(),
        table.grouped_aggregations_any(),
    );

    let root = roots
        .iter()
        .filter_map(|&index| model.row(index))
        .map(|row| row.id.as_ref().to_string())
        .collect::<Vec<_>>();

    let flat = if table.state().sorting.is_empty() {
        model
            .flat_rows()
            .iter()
            .filter_map(|&index| model.row(index))
            .map(|row| row.id.as_ref().to_string())
            .collect::<Vec<_>>()
    } else {
        fn walk_sorted(
            model: &fret_ui_headless::table::GroupedRowModel,
            indices: &[GroupedRowIndex],
            sorting: &[fret_ui_headless::table::SortSpec],
            columns: &[ColumnDef<FixtureRow>],
            data: &[FixtureRow],
            row_index_by_key: &HashMap<RowKey, usize>,
            group_aggs_u64: &HashMap<RowKey, std::sync::Arc<[(std::sync::Arc<str>, u64)]>>,
            group_aggs_any: &HashMap<
                RowKey,
                std::sync::Arc<[(std::sync::Arc<str>, fret_ui_headless::table::TanStackValue)]>,
            >,
            out: &mut Vec<String>,
        ) {
            for &index in indices {
                let Some(row) = model.row(index) else {
                    continue;
                };
                out.push(row.id.as_ref().to_string());
                if row.sub_rows.is_empty() {
                    continue;
                }
                let mut children = row.sub_rows.clone();
                sort_grouped_row_indices_in_place(
                    model,
                    children.as_mut_slice(),
                    sorting,
                    columns,
                    data,
                    row_index_by_key,
                    group_aggs_u64,
                    group_aggs_any,
                );
                walk_sorted(
                    model,
                    children.as_slice(),
                    sorting,
                    columns,
                    data,
                    row_index_by_key,
                    group_aggs_u64,
                    group_aggs_any,
                    out,
                );
            }
        }

        let mut out = Vec::new();
        walk_sorted(
            model,
            roots.as_slice(),
            table.state().sorting.as_slice(),
            columns,
            data,
            &row_index_by_key,
            table.grouped_u64_aggregations(),
            table.grouped_aggregations_any(),
            &mut out,
        );
        out
    };

    SortedGroupedIdsSnapshot { root, flat }
}

#[test]
fn tanstack_v8_grouping_sorting_precedence_parity() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture_path = manifest_dir
        .join("tests")
        .join("fixtures")
        .join("tanstack")
        .join("v8")
        .join("grouping_sorting_precedence.json");
    let fixture: Fixture =
        serde_json::from_str(&std::fs::read_to_string(&fixture_path).expect("fixture file"))
            .expect("fixture json");

    assert_eq!(fixture.case_id, "grouping_sorting_precedence");

    let data = fixture.data;
    let columns: Vec<ColumnDef<FixtureRow>> = vec![
        ColumnDef::<FixtureRow>::new("role").facet_key_by(|row: &FixtureRow| row.role),
        ColumnDef::<FixtureRow>::new("score_sum")
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.score as f64))
            .aggregation_fn_builtin(BuiltInAggregationFn::Sum),
        ColumnDef::<FixtureRow>::new("score_mean")
            .sort_value_by(|row: &FixtureRow| TanStackValue::Number(row.score as f64))
            .aggregation_fn_builtin(BuiltInAggregationFn::Mean),
    ];

    for snap in fixture.snapshots {
        let tanstack_options =
            TanStackTableOptions::from_json(&snap.options).expect("tanstack options");
        let options = tanstack_options.to_table_options();

        let tanstack_state = TanStackTableState::from_json(&snap.state).expect("tanstack state");
        let state = tanstack_state.to_table_state().expect("state conversion");

        let table = Table::builder(&data)
            .columns(columns.clone())
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
            .state(state)
            .options(options)
            .build();

        let actual = snapshot_sorted_grouped_ids(&table, columns.as_slice(), data.as_slice());
        assert_eq!(
            actual.root, snap.expect.sorted_grouped_ids.root,
            "snapshot {} sorted_grouped_ids.root mismatch",
            snap.id
        );
        assert_eq!(
            actual.flat, snap.expect.sorted_grouped_ids.flat,
            "snapshot {} sorted_grouped_ids.flat mismatch",
            snap.id
        );
    }
}
