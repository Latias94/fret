use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    ColumnDef, ColumnId, GroupedRowIndex, GroupedRowKind, GroupedRowModel, RowKey, SortSpec,
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

pub fn sort_grouped_row_indices_in_place<TData>(
    model: &GroupedRowModel,
    indices: &mut [GroupedRowIndex],
    sorting: &[SortSpec],
    columns: &[ColumnDef<TData>],
    data: &[TData],
    row_index_by_key: &HashMap<RowKey, usize>,
    group_aggs_u64: &HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
) {
    let Some(spec) = sorting.first() else {
        return;
    };
    let Some(col) = columns
        .iter()
        .find(|c| c.id.as_ref() == spec.column.as_ref())
    else {
        return;
    };

    indices.sort_by(|&a, &b| {
        let ra = model.row(a);
        let rb = model.row(b);
        let (Some(ra), Some(rb)) = (ra, rb) else {
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
            let extract_u64 = col
                .value_u64_fn
                .as_ref()
                .or_else(|| col.facet_key_fn.as_ref());
            if let Some(extract_u64) = extract_u64 {
                let key_a = representative_leaf_key(&ra.kind);
                let key_b = representative_leaf_key(&rb.kind);
                let idx_a = key_a.and_then(|k| row_index_by_key.get(&k).copied());
                let idx_b = key_b.and_then(|k| row_index_by_key.get(&k).copied());
                if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b) {
                    let va = extract_u64(&data[idx_a]);
                    let vb = extract_u64(&data[idx_b]);
                    ord = Some(va.cmp(&vb));
                }
            }
        }

        if ord.is_none() && col.sort_cmp.is_some() {
            let key_a = representative_leaf_key(&ra.kind);
            let key_b = representative_leaf_key(&rb.kind);
            let idx_a = key_a.and_then(|k| row_index_by_key.get(&k).copied());
            let idx_b = key_b.and_then(|k| row_index_by_key.get(&k).copied());
            if let (Some(idx_a), Some(idx_b)) = (idx_a, idx_b) {
                if let Some(cmp) = col.sort_cmp.as_ref() {
                    ord = Some(cmp(&data[idx_a], &data[idx_b]));
                }
            }
        }

        let mut ord = ord.unwrap_or_else(|| ra.key.cmp(&rb.key));
        if spec.desc {
            ord = ord.reverse();
        }
        ord
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::table::{Aggregation, Table, TableState};

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

        let mut row_index_by_key: HashMap<RowKey, usize> = HashMap::new();
        for i in 0..data.len() {
            row_index_by_key.insert(RowKey(i as u64), i);
        }
        let aggs = crate::headless::table::compute_grouped_u64_aggregations(
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
            &aggs,
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

        let mut row_index_by_key: HashMap<RowKey, usize> = HashMap::new();
        for i in 0..data.len() {
            row_index_by_key.insert(RowKey(i as u64), i);
        }
        let aggs = crate::headless::table::compute_grouped_u64_aggregations(
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
            &aggs,
        );

        let key_by_role = group_key_by_value(&grouped);
        let first = grouped.row(roots[0]).unwrap().key;
        let second = grouped.row(roots[1]).unwrap().key;
        assert_eq!(first, key_by_role[&1]);
        assert_eq!(second, key_by_role[&2]);
    }
}
