use std::collections::HashMap;
use std::sync::Arc;

use super::{
    ColumnDef, ColumnFilter, FilterFnDef, FilteringFnSpec, GlobalFilterState, RowModel,
    TableOptions,
};

pub type FacetKey = u64;
pub type FacetCounts = HashMap<FacetKey, usize>;

pub type FacetLabels<'a> = HashMap<FacetKey, &'a str>;

pub fn faceted_row_model_excluding<'a, TData>(
    pre_filtered: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    column_filters: &[ColumnFilter],
    global_filter: GlobalFilterState,
    options: TableOptions,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    global_filter_fn: &FilteringFnSpec,
    get_column_can_global_filter: Option<&dyn Fn(&ColumnDef<TData>, &TData) -> bool>,
    exclude_column_id: Option<&str>,
) -> RowModel<'a, TData> {
    let other_filters: Vec<ColumnFilter> = column_filters
        .iter()
        .filter(|f| exclude_column_id.is_none_or(|id| f.column.as_ref() != id))
        .cloned()
        .collect();
    super::filter_row_model(
        pre_filtered,
        columns,
        &other_filters,
        global_filter,
        options,
        filter_fns,
        global_filter_fn,
        get_column_can_global_filter,
    )
}

pub fn faceted_unique_values<'a, TData>(
    row_model: &RowModel<'a, TData>,
    column: &ColumnDef<TData>,
) -> FacetCounts {
    let Some(facet_key_fn) = column.facet_key_fn.as_ref() else {
        return FacetCounts::new();
    };

    let mut counts: FacetCounts = FacetCounts::new();
    for &row_index in row_model.flat_rows() {
        let Some(row) = row_model.row(row_index) else {
            continue;
        };
        let key = (facet_key_fn)(row.original);
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

pub fn faceted_unique_value_labels<'a, TData>(
    row_model: &RowModel<'a, TData>,
    column: &ColumnDef<TData>,
) -> FacetLabels<'a> {
    let Some(facet_key_fn) = column.facet_key_fn.as_ref() else {
        return FacetLabels::new();
    };
    let Some(facet_str_fn) = column.facet_str_fn.as_ref() else {
        return FacetLabels::new();
    };

    let mut labels: FacetLabels<'a> = FacetLabels::new();
    for &row_index in row_model.flat_rows() {
        let Some(row) = row_model.row(row_index) else {
            continue;
        };
        let key = (facet_key_fn)(row.original);
        labels
            .entry(key)
            .or_insert_with(|| (facet_str_fn)(row.original));
    }
    labels
}

pub fn faceted_min_max_u64<'a, TData>(
    row_model: &RowModel<'a, TData>,
    column: &ColumnDef<TData>,
) -> Option<(u64, u64)> {
    let Some(facet_key_fn) = column.facet_key_fn.as_ref() else {
        return None;
    };

    let mut iter = row_model
        .flat_rows()
        .iter()
        .filter_map(|&i| row_model.row(i).map(|r| (facet_key_fn)(r.original)));

    let first = iter.next()?;
    let mut min = first;
    let mut max = first;
    for v in iter {
        min = min.min(v);
        max = max.max(v);
    }
    Some((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{ColumnDef, ColumnFilter, Table, TableState, create_column_helper};
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct Item {
        status_key: u64,
        status_label: Arc<str>,
        role_key: u64,
        role_label: Arc<str>,
    }

    fn build_table(state: TableState) -> Table<'static, Item> {
        let data: &'static [Item] = Box::leak(
            vec![
                Item {
                    status_key: 1,
                    status_label: "A".into(),
                    role_key: 10,
                    role_label: "X".into(),
                },
                Item {
                    status_key: 2,
                    status_label: "B".into(),
                    role_key: 10,
                    role_label: "X".into(),
                },
                Item {
                    status_key: 1,
                    status_label: "A".into(),
                    role_key: 20,
                    role_label: "Y".into(),
                },
            ]
            .into_boxed_slice(),
        );

        let helper = create_column_helper::<Item>();
        let status = ColumnDef::new("status")
            .filter_by(|it: &Item, q| it.status_label.as_ref() == q)
            .facet_key_by(|it| it.status_key)
            .facet_str_by(|it| it.status_label.as_ref());
        let role = helper
            .accessor("role", |it| it.role_key)
            .filter_by(|it: &Item, q| it.role_label.as_ref() == q)
            .facet_key_by(|it| it.role_key)
            .facet_str_by(|it| it.role_label.as_ref());

        Table::builder(data)
            .columns(vec![status, role])
            .state(state)
            .build()
    }

    #[test]
    fn faceted_row_model_excludes_own_column_filter() {
        let mut state = TableState::default();
        state.column_filters = vec![
            ColumnFilter {
                column: "status".into(),
                value: serde_json::Value::from("A"),
            },
            ColumnFilter {
                column: "role".into(),
                value: serde_json::Value::from("X"),
            },
        ];
        state.global_filter = None;

        let table = build_table(state);

        // exclude status => only role=X applies => rows 0 and 1 remain
        let model = faceted_row_model_excluding(
            table.pre_filtered_row_model(),
            table.columns(),
            &table.state().column_filters,
            table.state().global_filter.clone(),
            TableOptions::default(),
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
            Some("status"),
        );
        let counts = faceted_unique_values(&model, table.column("status").unwrap());
        assert_eq!(counts.get(&1).copied(), Some(1));
        assert_eq!(counts.get(&2).copied(), Some(1));

        let labels = faceted_unique_value_labels(&model, table.column("status").unwrap());
        assert_eq!(labels.get(&1).copied(), Some("A"));
        assert_eq!(labels.get(&2).copied(), Some("B"));
    }

    #[test]
    fn faceted_min_max_uses_flat_rows() {
        let mut state = TableState::default();
        state.column_filters = Vec::new();
        state.global_filter = None;

        let table = build_table(state);
        let model = table.pre_filtered_row_model();
        let (min, max) = faceted_min_max_u64(model, table.column("status").unwrap()).unwrap();
        assert_eq!((min, max), (1, 2));
    }
}
