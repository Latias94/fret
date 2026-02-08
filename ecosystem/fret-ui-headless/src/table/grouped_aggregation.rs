use std::collections::HashMap;
use std::sync::Arc;

use super::{Aggregation, ColumnDef, ColumnId, GroupedRowKind, GroupedRowModel, RowKey, RowModel};

pub fn compute_grouped_u64_aggregations<TData>(
    model: &GroupedRowModel,
    data: &[TData],
    row_index_by_key: &HashMap<RowKey, usize>,
    agg_columns: &[&ColumnDef<TData>],
) -> HashMap<RowKey, Arc<[(ColumnId, u64)]>> {
    if agg_columns.is_empty() || model.flat_rows().is_empty() {
        return Default::default();
    }

    let col_count = agg_columns.len();

    let idx = |node: usize, col_i: usize| node * col_count + col_i;

    let mut postorder: Vec<usize> = Vec::new();
    let mut visited: std::collections::HashSet<usize> = Default::default();
    let mut max_index: usize = 0;

    fn push_postorder(
        model: &GroupedRowModel,
        node: usize,
        visited: &mut std::collections::HashSet<usize>,
        out: &mut Vec<usize>,
        max_index: &mut usize,
    ) {
        if !visited.insert(node) {
            return;
        }
        *max_index = (*max_index).max(node);

        let Some(row) = model.row(node) else {
            return;
        };
        for &child in &row.sub_rows {
            push_postorder(model, child, visited, out, max_index);
        }
        out.push(node);
    }

    for &root in model.root_rows() {
        push_postorder(model, root, &mut visited, &mut postorder, &mut max_index);
    }

    let node_count = max_index + 1;

    let mut leaf_count: Vec<usize> = vec![0; node_count];
    let mut sum: Vec<Option<u64>> = vec![None; node_count * col_count];
    let mut min: Vec<Option<u64>> = vec![None; node_count * col_count];
    let mut max: Vec<Option<u64>> = vec![None; node_count * col_count];

    for node in postorder {
        let Some(row) = model.row(node) else {
            continue;
        };

        match &row.kind {
            GroupedRowKind::Leaf { row_key } => {
                leaf_count[node] = 1;
                let Some(&data_index) = row_index_by_key.get(row_key) else {
                    continue;
                };
                let item = &data[data_index];

                for (col_i, col) in agg_columns.iter().enumerate() {
                    let extract_u64 = col
                        .value_u64_fn
                        .as_ref()
                        .or_else(|| col.facet_key_fn.as_ref());
                    let Some(extract_u64) = extract_u64 else {
                        continue;
                    };

                    let v = extract_u64(item);
                    sum[idx(node, col_i)] = Some(v);
                    min[idx(node, col_i)] = Some(v);
                    max[idx(node, col_i)] = Some(v);
                }
            }
            GroupedRowKind::Group { leaf_row_count, .. } => {
                leaf_count[node] = *leaf_row_count;
                for &child in &row.sub_rows {
                    for col_i in 0..col_count {
                        if let Some(child_sum) = sum[idx(child, col_i)] {
                            let next = match sum[idx(node, col_i)] {
                                Some(acc) => acc.checked_add(child_sum),
                                None => Some(child_sum),
                            };
                            sum[idx(node, col_i)] = next;
                        }

                        if let Some(child_min) = min[idx(child, col_i)] {
                            let next = match min[idx(node, col_i)] {
                                Some(acc) => Some(acc.min(child_min)),
                                None => Some(child_min),
                            };
                            min[idx(node, col_i)] = next;
                        }

                        if let Some(child_max) = max[idx(child, col_i)] {
                            let next = match max[idx(node, col_i)] {
                                Some(acc) => Some(acc.max(child_max)),
                                None => Some(child_max),
                            };
                            max[idx(node, col_i)] = next;
                        }
                    }
                }
            }
        }
    }

    let mut out_u64: HashMap<RowKey, Arc<[(ColumnId, u64)]>> = Default::default();
    for &node in visited.iter() {
        let Some(row) = model.row(node) else {
            continue;
        };
        if !matches!(row.kind, GroupedRowKind::Group { .. }) {
            continue;
        }

        let denom = leaf_count[node] as u64;
        let mut u64_values: Vec<(ColumnId, u64)> = Vec::new();
        for (col_i, col) in agg_columns.iter().enumerate() {
            let value = match col.aggregation {
                Aggregation::None => None,
                Aggregation::Count => Some(denom),
                Aggregation::SumU64 => sum[idx(node, col_i)],
                Aggregation::MinU64 => min[idx(node, col_i)],
                Aggregation::MaxU64 => max[idx(node, col_i)],
                Aggregation::MeanU64 => {
                    if denom == 0 {
                        None
                    } else {
                        sum[idx(node, col_i)].map(|s| s / denom)
                    }
                }
            };

            let Some(v) = value else {
                continue;
            };
            u64_values.push((col.id.clone(), v));
        }

        out_u64.insert(row.key, Arc::from(u64_values.into_boxed_slice()));
    }

    out_u64
}

pub fn compute_grouped_u64_aggregations_from_core<'a, TData>(
    model: &GroupedRowModel,
    core: &RowModel<'a, TData>,
    agg_columns: &[&ColumnDef<TData>],
) -> HashMap<RowKey, Arc<[(ColumnId, u64)]>> {
    if agg_columns.is_empty() || model.flat_rows().is_empty() {
        return Default::default();
    }

    let col_count = agg_columns.len();

    let idx = |node: usize, col_i: usize| node * col_count + col_i;

    let mut postorder: Vec<usize> = Vec::new();
    let mut visited: std::collections::HashSet<usize> = Default::default();
    let mut max_index: usize = 0;

    fn push_postorder(
        model: &GroupedRowModel,
        node: usize,
        visited: &mut std::collections::HashSet<usize>,
        out: &mut Vec<usize>,
        max_index: &mut usize,
    ) {
        if !visited.insert(node) {
            return;
        }
        *max_index = (*max_index).max(node);

        let Some(row) = model.row(node) else {
            return;
        };
        for &child in &row.sub_rows {
            push_postorder(model, child, visited, out, max_index);
        }
        out.push(node);
    }

    for &root in model.root_rows() {
        push_postorder(model, root, &mut visited, &mut postorder, &mut max_index);
    }

    let node_count = max_index + 1;

    let mut leaf_count: Vec<usize> = vec![0; node_count];
    let mut sum: Vec<Option<u64>> = vec![None; node_count * col_count];
    let mut min: Vec<Option<u64>> = vec![None; node_count * col_count];
    let mut max: Vec<Option<u64>> = vec![None; node_count * col_count];

    for node in postorder {
        let Some(row) = model.row(node) else {
            continue;
        };

        match &row.kind {
            GroupedRowKind::Leaf { row_key } => {
                leaf_count[node] = 1;
                let Some(core_index) = core.row_by_key(*row_key) else {
                    continue;
                };
                let Some(item) = core.row(core_index).map(|r| r.original) else {
                    continue;
                };

                for (col_i, col) in agg_columns.iter().enumerate() {
                    let extract_u64 = col
                        .value_u64_fn
                        .as_ref()
                        .or_else(|| col.facet_key_fn.as_ref());
                    let Some(extract_u64) = extract_u64 else {
                        continue;
                    };

                    let v = extract_u64(item);
                    sum[idx(node, col_i)] = Some(v);
                    min[idx(node, col_i)] = Some(v);
                    max[idx(node, col_i)] = Some(v);
                }
            }
            GroupedRowKind::Group { leaf_row_count, .. } => {
                leaf_count[node] = *leaf_row_count;
                for &child in &row.sub_rows {
                    for col_i in 0..col_count {
                        if let Some(child_sum) = sum[idx(child, col_i)] {
                            let next = match sum[idx(node, col_i)] {
                                Some(acc) => acc.checked_add(child_sum),
                                None => Some(child_sum),
                            };
                            sum[idx(node, col_i)] = next;
                        }

                        if let Some(child_min) = min[idx(child, col_i)] {
                            let next = match min[idx(node, col_i)] {
                                Some(acc) => Some(acc.min(child_min)),
                                None => Some(child_min),
                            };
                            min[idx(node, col_i)] = next;
                        }

                        if let Some(child_max) = max[idx(child, col_i)] {
                            let next = match max[idx(node, col_i)] {
                                Some(acc) => Some(acc.max(child_max)),
                                None => Some(child_max),
                            };
                            max[idx(node, col_i)] = next;
                        }
                    }
                }
            }
        }
    }

    let mut out_u64: HashMap<RowKey, Arc<[(ColumnId, u64)]>> = Default::default();
    for &node in visited.iter() {
        let Some(row) = model.row(node) else {
            continue;
        };
        if !matches!(row.kind, GroupedRowKind::Group { .. }) {
            continue;
        }

        let denom = leaf_count[node] as u64;
        let mut u64_values: Vec<(ColumnId, u64)> = Vec::new();
        for (col_i, col) in agg_columns.iter().enumerate() {
            let value = match col.aggregation {
                Aggregation::None => None,
                Aggregation::Count => Some(denom),
                Aggregation::SumU64 => sum[idx(node, col_i)],
                Aggregation::MinU64 => min[idx(node, col_i)],
                Aggregation::MaxU64 => max[idx(node, col_i)],
                Aggregation::MeanU64 => {
                    if denom == 0 {
                        None
                    } else {
                        sum[idx(node, col_i)].map(|s| s / denom)
                    }
                }
            };

            let Some(v) = value else {
                continue;
            };
            u64_values.push((col.id.clone(), v));
        }

        out_u64.insert(row.key, Arc::from(u64_values.into_boxed_slice()));
    }

    out_u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{ColumnDef, Table, TableState};
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct Item {
        role: u64,
        score: u64,
    }

    fn agg_value(
        map: &HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
        key: RowKey,
        col: &str,
    ) -> Option<u64> {
        map.get(&key)
            .and_then(|entries| entries.iter().find(|(id, _)| id.as_ref() == col))
            .map(|(_, v)| *v)
    }

    #[test]
    fn grouped_u64_aggregation_sums_by_group() {
        let data = vec![
            Item { role: 1, score: 10 },
            Item { role: 2, score: 7 },
            Item { role: 1, score: 5 },
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

        let agg_columns: Vec<&ColumnDef<Item>> = columns
            .iter()
            .filter(|c| c.aggregation != Aggregation::None)
            .collect();
        let aggs =
            compute_grouped_u64_aggregations(&grouped, &data, &row_index_by_key, &agg_columns);

        let mut key_by_role: HashMap<u64, RowKey> = HashMap::new();
        for &root in grouped.root_rows() {
            let row = grouped.row(root).unwrap();
            let GroupedRowKind::Group { grouping_value, .. } = row.kind else {
                continue;
            };
            key_by_role.insert(grouping_value, row.key);
        }

        assert_eq!(agg_value(&aggs, key_by_role[&1], "score"), Some(15));
        assert_eq!(agg_value(&aggs, key_by_role[&2], "score"), Some(7));
    }

    #[test]
    fn grouped_u64_aggregation_count_does_not_require_value_extractor() {
        let data = vec![Item { role: 1, score: 10 }, Item { role: 1, score: 5 }];

        let columns = vec![
            ColumnDef::new("role").facet_key_by(|it: &Item| it.role),
            ColumnDef::new("score").aggregate(Aggregation::Count),
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

        let agg_columns: Vec<&ColumnDef<Item>> = columns
            .iter()
            .filter(|c| c.aggregation != Aggregation::None)
            .collect();
        let aggs =
            compute_grouped_u64_aggregations(&grouped, &data, &row_index_by_key, &agg_columns);

        let root = grouped.root_rows()[0];
        let row = grouped.row(root).unwrap();
        assert!(matches!(row.kind, GroupedRowKind::Group { .. }));
        assert_eq!(agg_value(&aggs, row.key, "score"), Some(2));
    }
}
