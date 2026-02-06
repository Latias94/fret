use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    ColumnDef, ColumnId, GroupedRowIndex, GroupedRowKind, GroupedRowModel, RowKey, SortSpec,
    TanStackValue,
};

fn representative_leaf_key(kind: &GroupedRowKind) -> Option<RowKey> {
    match kind {
        GroupedRowKind::Group {
            first_leaf_row_key, ..
        } => Some(*first_leaf_row_key),
        GroupedRowKind::Leaf { row_key } => Some(*row_key),
    }
}

fn grouped_u64_value(
    group_aggs_u64: &HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
    row_key: RowKey,
    col_id: &ColumnId,
) -> Option<u64> {
    group_aggs_u64.get(&row_key).and_then(|entries| {
        entries
            .iter()
            .find(|entry| entry.0.as_ref() == col_id.as_ref())
            .map(|entry| entry.1)
    })
}

fn grouped_any_value<'a>(
    group_aggs_any: &'a HashMap<RowKey, Arc<[(ColumnId, TanStackValue)]>>,
    row_key: RowKey,
    col_id: &ColumnId,
) -> Option<&'a TanStackValue> {
    group_aggs_any.get(&row_key).and_then(|entries| {
        entries
            .iter()
            .find(|entry| entry.0.as_ref() == col_id.as_ref())
            .map(|entry| &entry.1)
    })
}

fn to_number_for_compare(value: &TanStackValue) -> f64 {
    match value {
        TanStackValue::Undefined => f64::NAN,
        TanStackValue::Null => 0.0,
        TanStackValue::Bool(v) => {
            if *v {
                1.0
            } else {
                0.0
            }
        }
        TanStackValue::Number(v) | TanStackValue::DateTime(v) => *v,
        TanStackValue::String(v) => v.parse::<f64>().unwrap_or(f64::NAN),
        TanStackValue::Array(v) => {
            if v.is_empty() {
                0.0
            } else {
                f64::NAN
            }
        }
    }
}

fn strict_equal(a: &TanStackValue, b: &TanStackValue) -> bool {
    match (a, b) {
        (TanStackValue::Undefined, TanStackValue::Undefined)
        | (TanStackValue::Null, TanStackValue::Null) => true,
        (TanStackValue::Bool(a), TanStackValue::Bool(b)) => a == b,
        (TanStackValue::Number(a), TanStackValue::Number(b))
        | (TanStackValue::DateTime(a), TanStackValue::DateTime(b)) => {
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a == b
            }
        }
        (TanStackValue::String(a), TanStackValue::String(b)) => a == b,
        _ => false,
    }
}

fn abstract_gt(a: &TanStackValue, b: &TanStackValue) -> bool {
    match (a, b) {
        (TanStackValue::String(a), TanStackValue::String(b)) => a.as_ref() > b.as_ref(),
        _ => {
            let a = to_number_for_compare(a);
            let b = to_number_for_compare(b);
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a > b
            }
        }
    }
}

fn compare_basic_tanstack(a: &TanStackValue, b: &TanStackValue) -> Ordering {
    if strict_equal(a, b) {
        return Ordering::Equal;
    }
    if abstract_gt(a, b) {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

fn row_data_index(
    kind: &GroupedRowKind,
    row_index_by_key: &HashMap<RowKey, usize>,
) -> Option<usize> {
    let key = representative_leaf_key(kind)?;
    row_index_by_key.get(&key).copied()
}

fn compare_for_spec<TData>(
    model: &GroupedRowModel,
    a: GroupedRowIndex,
    b: GroupedRowIndex,
    col: &ColumnDef<TData>,
    spec: &SortSpec,
    data: &[TData],
    row_index_by_key: &HashMap<RowKey, usize>,
    group_aggs_u64: &HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
    group_aggs_any: &HashMap<RowKey, Arc<[(ColumnId, TanStackValue)]>>,
) -> Ordering {
    let Some(ra) = model.row(a) else {
        return Ordering::Equal;
    };
    let Some(rb) = model.row(b) else {
        return Ordering::Equal;
    };

    let mut ord: Option<Ordering> = None;

    if let (
        GroupedRowKind::Group {
            grouping_column: ca,
            grouping_value: va,
            ..
        },
        GroupedRowKind::Group {
            grouping_column: cb,
            grouping_value: vb,
            ..
        },
    ) = (&ra.kind, &rb.kind)
        && ca.as_ref() == col.id.as_ref()
        && cb.as_ref() == col.id.as_ref()
    {
        ord = Some(va.cmp(vb));
    }

    if ord.is_none() {
        let va = grouped_u64_value(group_aggs_u64, ra.key, &col.id);
        let vb = grouped_u64_value(group_aggs_u64, rb.key, &col.id);
        ord = match (va, vb) {
            (Some(va), Some(vb)) => Some(va.cmp(&vb)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => None,
        };
    }

    if ord.is_none() {
        let va = grouped_any_value(group_aggs_any, ra.key, &col.id);
        let vb = grouped_any_value(group_aggs_any, rb.key, &col.id);
        ord = match (va, vb) {
            (Some(va), Some(vb)) => Some(compare_basic_tanstack(va, vb)),
            (Some(_), None) => Some(Ordering::Less),
            (None, Some(_)) => Some(Ordering::Greater),
            (None, None) => None,
        };
    }

    let both_leaf = matches!(ra.kind, GroupedRowKind::Leaf { .. })
        && matches!(rb.kind, GroupedRowKind::Leaf { .. });

    if ord.is_none() && both_leaf {
        let extract_u64 = col
            .value_u64_fn
            .as_ref()
            .or_else(|| col.facet_key_fn.as_ref());
        if let Some(extract_u64) = extract_u64 {
            let idx_a = row_data_index(&ra.kind, row_index_by_key);
            let idx_b = row_data_index(&rb.kind, row_index_by_key);
            if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b) {
                let va = extract_u64(&data[idx_a]);
                let vb = extract_u64(&data[idx_b]);
                ord = Some(va.cmp(&vb));
            }
        }
    }

    if ord.is_none() && both_leaf && col.sort_cmp.is_some() {
        let idx_a = row_data_index(&ra.kind, row_index_by_key);
        let idx_b = row_data_index(&rb.kind, row_index_by_key);
        if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b)
            && let Some(cmp) = col.sort_cmp.as_ref()
        {
            ord = Some(cmp(&data[idx_a], &data[idx_b]));
        }
    }

    if ord.is_none() && both_leaf {
        if let Some(get_value) = col.sort_value.as_ref() {
            let idx_a = row_data_index(&ra.kind, row_index_by_key);
            let idx_b = row_data_index(&rb.kind, row_index_by_key);
            if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b) {
                let va = get_value(&data[idx_a]);
                let vb = get_value(&data[idx_b]);
                ord = Some(compare_basic_tanstack(&va, &vb));
            }
        }
    }

    let mut ord = ord.unwrap_or(Ordering::Equal);
    if ord != Ordering::Equal {
        if spec.desc {
            ord = ord.reverse();
        }
        if col.invert_sorting {
            ord = ord.reverse();
        }
    }
    ord
}

pub fn sort_grouped_row_indices_in_place<TData>(
    model: &GroupedRowModel,
    indices: &mut [GroupedRowIndex],
    sorting: &[SortSpec],
    columns: &[ColumnDef<TData>],
    data: &[TData],
    row_index_by_key: &HashMap<RowKey, usize>,
    group_aggs_u64: &HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
    group_aggs_any: &HashMap<RowKey, Arc<[(ColumnId, TanStackValue)]>>,
) {
    if sorting.is_empty() {
        return;
    }

    indices.sort_by(|&a, &b| {
        let ra = model.row(a);
        let rb = model.row(b);
        let (Some(ra), Some(rb)) = (ra, rb) else {
            return Ordering::Equal;
        };

        for spec in sorting {
            let Some(col) = columns
                .iter()
                .find(|c| c.id.as_ref() == spec.column.as_ref())
            else {
                continue;
            };

            let ord = compare_for_spec(
                model,
                a,
                b,
                col,
                spec,
                data,
                row_index_by_key,
                group_aggs_u64,
                group_aggs_any,
            );
            if ord != Ordering::Equal {
                return ord;
            }
        }

        ra.key.cmp(&rb.key)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{
        Aggregation, BuiltInAggregationFn, Table, TableState, TanStackValue,
        compute_grouped_u64_aggregations,
    };

    #[derive(Debug, Clone)]
    struct Item {
        role: u64,
        score: u64,
    }

    fn group_key_by_value(model: &GroupedRowModel) -> HashMap<u64, RowKey> {
        let mut out = HashMap::new();
        for &root in model.root_rows() {
            let row = model.row(root).unwrap();
            let GroupedRowKind::Group { grouping_value, .. } = row.kind else {
                continue;
            };
            out.insert(grouping_value, row.key);
        }
        out
    }

    fn row_index_by_key<TData>(table: &Table<'_, TData>) -> HashMap<RowKey, usize> {
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

    fn agg_columns<TData>(cols: &[ColumnDef<TData>]) -> Vec<&ColumnDef<TData>> {
        cols.iter()
            .filter(|c| c.aggregation != Aggregation::None)
            .collect()
    }

    #[test]
    fn grouped_sort_by_grouping_value() {
        let data = vec![Item { role: 2, score: 1 }, Item { role: 1, score: 1 }];

        let columns = vec![
            ColumnDef::new("role").facet_key_by(|it: &Item| it.role),
            ColumnDef::new("score")
                .value_u64_by(|it: &Item| it.score)
                .aggregate(Aggregation::SumU64),
        ];

        let mut state = TableState::default();
        state.grouping = vec![Arc::from("role")];
        let table = Table::builder(&data)
            .columns(columns.clone())
            .state(state)
            .build();
        let grouped = table.grouped_row_model().clone();

        let row_index_by_key = row_index_by_key(&table);
        let aggs_u64 = compute_grouped_u64_aggregations(
            &grouped,
            &data,
            &row_index_by_key,
            &agg_columns(&columns),
        );

        let mut roots = grouped.root_rows().to_vec();
        sort_grouped_row_indices_in_place(
            &grouped,
            &mut roots,
            &[SortSpec {
                column: Arc::from("role"),
                desc: false,
            }],
            &columns,
            &data,
            &row_index_by_key,
            &aggs_u64,
            table.grouped_aggregations_any(),
        );

        let key_by_role = group_key_by_value(&grouped);
        let first = grouped.row(roots[0]).unwrap().key;
        let second = grouped.row(roots[1]).unwrap().key;
        assert_eq!(first, key_by_role[&1]);
        assert_eq!(second, key_by_role[&2]);
    }

    #[test]
    fn grouped_sort_by_aggregation() {
        let data = vec![
            Item { role: 1, score: 10 },
            Item { role: 2, score: 1 },
            Item { role: 1, score: 1 },
        ];

        let columns = vec![
            ColumnDef::new("role").facet_key_by(|it: &Item| it.role),
            ColumnDef::new("score")
                .value_u64_by(|it: &Item| it.score)
                .aggregate(Aggregation::SumU64),
        ];

        let mut state = TableState::default();
        state.grouping = vec![Arc::from("role")];
        let table = Table::builder(&data)
            .columns(columns.clone())
            .state(state)
            .build();
        let grouped = table.grouped_row_model().clone();

        let row_index_by_key = row_index_by_key(&table);
        let aggs_u64 = compute_grouped_u64_aggregations(
            &grouped,
            &data,
            &row_index_by_key,
            &agg_columns(&columns),
        );

        let mut roots = grouped.root_rows().to_vec();
        sort_grouped_row_indices_in_place(
            &grouped,
            &mut roots,
            &[SortSpec {
                column: Arc::from("score"),
                desc: true,
            }],
            &columns,
            &data,
            &row_index_by_key,
            &aggs_u64,
            table.grouped_aggregations_any(),
        );

        let key_by_role = group_key_by_value(&grouped);
        let first = grouped.row(roots[0]).unwrap().key;
        let second = grouped.row(roots[1]).unwrap().key;
        assert_eq!(first, key_by_role[&1]);
        assert_eq!(second, key_by_role[&2]);
    }

    #[test]
    fn grouped_sort_applies_secondary_spec_for_group_ties() {
        let data = vec![
            Item { role: 1, score: 1 },
            Item { role: 1, score: 9 },
            Item { role: 2, score: 4 },
            Item { role: 2, score: 3 },
            Item { role: 2, score: 3 },
        ];

        let columns = vec![
            ColumnDef::new("role").facet_key_by(|it: &Item| it.role),
            ColumnDef::new("score")
                .value_u64_by(|it: &Item| it.score)
                .aggregate(Aggregation::SumU64),
        ];

        let mut state = TableState::default();
        state.grouping = vec![Arc::from("role")];
        let table = Table::builder(&data)
            .columns(columns.clone())
            .state(state)
            .build();
        let grouped = table.grouped_row_model().clone();

        let row_index_by_key = row_index_by_key(&table);
        let aggs_u64 = compute_grouped_u64_aggregations(
            &grouped,
            &data,
            &row_index_by_key,
            &agg_columns(&columns),
        );

        let mut roots = grouped.root_rows().to_vec();
        sort_grouped_row_indices_in_place(
            &grouped,
            &mut roots,
            &[
                SortSpec {
                    column: Arc::from("score"),
                    desc: true,
                },
                SortSpec {
                    column: Arc::from("role"),
                    desc: true,
                },
            ],
            &columns,
            &data,
            &row_index_by_key,
            &aggs_u64,
            table.grouped_aggregations_any(),
        );

        let key_by_role = group_key_by_value(&grouped);
        let first = grouped.row(roots[0]).unwrap().key;
        let second = grouped.row(roots[1]).unwrap().key;
        assert_eq!(first, key_by_role[&2]);
        assert_eq!(second, key_by_role[&1]);
    }

    #[test]
    fn grouped_sort_uses_any_aggregation_values_for_groups() {
        let data = vec![
            Item { role: 1, score: 1 },
            Item { role: 1, score: 9 },
            Item { role: 2, score: 4 },
            Item { role: 2, score: 3 },
            Item { role: 2, score: 3 },
        ];

        let columns = vec![
            ColumnDef::new("role").facet_key_by(|it: &Item| it.role),
            ColumnDef::new("score_mean")
                .sort_value_by(|it: &Item| TanStackValue::Number(it.score as f64))
                .aggregation_fn_builtin(BuiltInAggregationFn::Mean),
        ];

        let mut state = TableState::default();
        state.grouping = vec![Arc::from("role")];
        let table = Table::builder(&data)
            .columns(columns.clone())
            .state(state)
            .build();
        let grouped = table.grouped_row_model().clone();

        let row_index_by_key = row_index_by_key(&table);
        let aggs_u64 = compute_grouped_u64_aggregations(
            &grouped,
            &data,
            &row_index_by_key,
            &agg_columns(&columns),
        );

        let mut roots = grouped.root_rows().to_vec();
        sort_grouped_row_indices_in_place(
            &grouped,
            &mut roots,
            &[SortSpec {
                column: Arc::from("score_mean"),
                desc: true,
            }],
            &columns,
            &data,
            &row_index_by_key,
            &aggs_u64,
            table.grouped_aggregations_any(),
        );

        let key_by_role = group_key_by_value(&grouped);
        let first = grouped.row(roots[0]).unwrap().key;
        let second = grouped.row(roots[1]).unwrap().key;
        assert_eq!(first, key_by_role[&1]);
        assert_eq!(second, key_by_role[&2]);
    }
}
