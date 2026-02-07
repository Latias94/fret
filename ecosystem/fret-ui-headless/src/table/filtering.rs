use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use super::{
    ColumnDef, ColumnId, FilteringFnSpec, RowIndex, RowKey, RowModel, SortValueFn, TableOptions,
    TanStackValue, column::BuiltInFilterFn,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnFilter {
    pub column: ColumnId,
    pub value: Value,
}

pub type ColumnFiltersState = Vec<ColumnFilter>;
/// TanStack-compatible global filter state (`any | undefined`).
///
/// Note: TanStack's default global filter UX typically uses a string, but upstream allows any
/// value and relies on the configured `globalFilterFn` to interpret it.
pub type GlobalFilterState = Option<Value>;

#[derive(Clone)]
pub enum FilterFnDef {
    BuiltIn(BuiltInFilterFn),
    Value(Arc<dyn Fn(&TanStackValue, &Value) -> bool>),
    ValueWithMeta(Arc<dyn Fn(&TanStackValue, &Value, &mut dyn FnMut(Value)) -> bool>),
}

pub type RowColumnFilters = HashMap<ColumnId, bool>;
pub type RowColumnFiltersMeta = HashMap<ColumnId, Value>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RowFilterStateSnapshot {
    pub filterable_ids: Vec<ColumnId>,
    pub row_column_filters: HashMap<RowKey, RowColumnFilters>,
    pub row_column_filters_meta: HashMap<RowKey, RowColumnFiltersMeta>,
}

/// TanStack-aligned column filter state transition for `column.setFilterValue(value)`.
///
/// Notes:
/// - This models the upstream state update behavior, including `autoRemove` rules for built-in
///   filter functions.
/// - `resolveFilterValue` is applied at row-model time (TanStack behavior) and does not mutate the
///   stored filter value.
pub fn set_column_filter_value_tanstack<TData>(
    column_filters: &mut ColumnFiltersState,
    data: &[TData],
    column: &ColumnDef<TData>,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    value: Value,
) {
    let existing_index = column_filters
        .iter()
        .position(|f| f.column.as_ref() == column.id.as_ref());

    let resolved = resolve_filter_fn_for_column(data, column, filter_fns);

    let should_remove = match resolved.as_ref() {
        None => is_json_filter_value_empty(&value),
        Some(ResolvedFilterBehavior::BuiltIn(builtin)) => {
            should_auto_remove_built_in_filter(*builtin, &value)
        }
        Some(ResolvedFilterBehavior::Custom) => is_json_filter_value_empty(&value),
    };

    if should_remove {
        if let Some(i) = existing_index {
            column_filters.remove(i);
        }
        return;
    }

    let next = ColumnFilter {
        column: column.id.clone(),
        value,
    };

    match existing_index {
        Some(i) => {
            if let Some(entry) = column_filters.get_mut(i) {
                *entry = next;
            }
        }
        None => column_filters.push(next),
    }
}

pub(crate) fn column_filters_updater_set_value_tanstack<TData>(
    data: &[TData],
    column: &ColumnDef<TData>,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    value: Value,
) -> super::Updater<ColumnFiltersState> {
    let column_id = column.id.clone();
    let auto_remove_behavior = resolve_filter_fn_for_column(data, column, filter_fns);

    super::Updater::Func(Arc::new(move |old| {
        let mut next = old.clone();
        let existing_index = next
            .iter()
            .position(|f| f.column.as_ref() == column_id.as_ref());

        let should_remove = match auto_remove_behavior.as_ref() {
            None => is_json_filter_value_empty(&value),
            Some(ResolvedFilterBehavior::BuiltIn(builtin)) => {
                should_auto_remove_built_in_filter(*builtin, &value)
            }
            Some(ResolvedFilterBehavior::Custom) => is_json_filter_value_empty(&value),
        };

        if should_remove {
            if let Some(i) = existing_index {
                next.remove(i);
            }
            return next;
        }

        let entry = ColumnFilter {
            column: column_id.clone(),
            value: value.clone(),
        };

        match existing_index {
            Some(i) => {
                if let Some(existing) = next.get_mut(i) {
                    *existing = entry;
                }
            }
            None => next.push(entry),
        }

        next
    }))
}

pub(crate) fn global_filter_updater_set_value_tanstack(
    value: GlobalFilterState,
) -> super::Updater<GlobalFilterState> {
    super::Updater::Value(value)
}

pub fn evaluate_row_filter_state<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    column_filters: &[ColumnFilter],
    global_filter: GlobalFilterState,
    options: TableOptions,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    global_filter_fn: &FilteringFnSpec,
    get_column_can_global_filter: Option<&dyn Fn(&ColumnDef<TData>, &TData) -> bool>,
) -> RowFilterStateSnapshot {
    if row_model.root_rows().is_empty() {
        return RowFilterStateSnapshot::default();
    }

    #[derive(Clone)]
    enum ResolvedFilterFn<TData> {
        Custom(super::FilterFn<TData>),
        CustomWithMeta(super::FilterFnWithMeta<TData>),
        BuiltIn {
            builtin: BuiltInFilterFn,
            value: SortValueFn<TData>,
        },
        Value {
            value: SortValueFn<TData>,
            f: Arc<dyn Fn(&TanStackValue, &Value) -> bool>,
        },
        ValueWithMeta {
            value: SortValueFn<TData>,
            f: Arc<dyn Fn(&TanStackValue, &Value, &mut dyn FnMut(Value)) -> bool>,
        },
    }

    impl<TData> ResolvedFilterFn<TData> {
        fn resolve_filter_value(&self, value: &Value) -> Value {
            match self {
                Self::BuiltIn { builtin, .. } => match builtin {
                    BuiltInFilterFn::InNumberRange => resolve_in_number_range(value),
                    _ => value.clone(),
                },
                _ => value.clone(),
            }
        }

        fn apply_with_meta(
            &self,
            row: &TData,
            _column_id: &ColumnId,
            filter_value: &Value,
            add_meta: &mut dyn FnMut(Value),
        ) -> bool {
            match self {
                Self::Custom(f) => f(row, filter_value),
                Self::CustomWithMeta(f) => f(row, filter_value, add_meta),
                Self::BuiltIn { builtin, value } => {
                    let v = value(row);
                    apply_built_in_filter_fn(*builtin, &v, filter_value)
                }
                Self::Value { value, f } => {
                    let v = value(row);
                    f(&v, filter_value)
                }
                Self::ValueWithMeta { value, f } => {
                    let v = value(row);
                    f(&v, filter_value, add_meta)
                }
            }
        }
    }

    fn resolve_column_filter_fn<TData>(
        row_model: &RowModel<'_, TData>,
        column: &ColumnDef<TData>,
        filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    ) -> Option<ResolvedFilterFn<TData>> {
        if let Some(custom) = column.filter_fn_with_meta.as_ref() {
            return Some(ResolvedFilterFn::CustomWithMeta(custom.clone()));
        }
        if let Some(custom) = column.filter_fn.as_ref() {
            return Some(ResolvedFilterFn::Custom(custom.clone()));
        }

        let spec = column.filtering_fn.as_ref()?;
        match spec {
            FilteringFnSpec::Auto => {
                let first_row = row_model
                    .flat_rows()
                    .first()
                    .and_then(|&i| row_model.row(i));
                let value = match (first_row, column.sort_value.as_ref()) {
                    (Some(row), Some(get_value)) => get_value(row.original),
                    _ => TanStackValue::Undefined,
                };

                let builtin = match value {
                    TanStackValue::String(_) => BuiltInFilterFn::IncludesString,
                    TanStackValue::Number(_) => BuiltInFilterFn::InNumberRange,
                    TanStackValue::Bool(_) => BuiltInFilterFn::Equals,
                    TanStackValue::DateTime(_) => BuiltInFilterFn::Equals,
                    TanStackValue::Array(_) => BuiltInFilterFn::Equals,
                    _ => BuiltInFilterFn::WeakEquals,
                };

                let value = column.sort_value.as_ref()?.clone();
                Some(ResolvedFilterFn::BuiltIn { builtin, value })
            }
            FilteringFnSpec::BuiltIn(builtin) => {
                let value = column.sort_value.as_ref()?.clone();
                Some(ResolvedFilterFn::BuiltIn {
                    builtin: *builtin,
                    value,
                })
            }
            FilteringFnSpec::Named(key) => match filter_fns.get(key) {
                Some(FilterFnDef::BuiltIn(builtin)) => {
                    let value = column.sort_value.as_ref()?.clone();
                    Some(ResolvedFilterFn::BuiltIn {
                        builtin: *builtin,
                        value,
                    })
                }
                Some(FilterFnDef::Value(f)) => {
                    let value = column.sort_value.as_ref()?.clone();
                    Some(ResolvedFilterFn::Value {
                        value,
                        f: f.clone(),
                    })
                }
                Some(FilterFnDef::ValueWithMeta(f)) => {
                    let value = column.sort_value.as_ref()?.clone();
                    Some(ResolvedFilterFn::ValueWithMeta {
                        value,
                        f: f.clone(),
                    })
                }
                None => {
                    let builtin = builtin_filter_key(key.as_ref())?;
                    let value = column.sort_value.as_ref()?.clone();
                    Some(ResolvedFilterFn::BuiltIn { builtin, value })
                }
            },
        }
    }

    #[derive(Clone)]
    struct ResolvedColumnFilter<TData> {
        column_id: ColumnId,
        f: ResolvedFilterFn<TData>,
        resolved_value: Value,
    }

    #[derive(Clone)]
    struct ResolvedGlobalFilter<TData> {
        column_id: ColumnId,
        f: ResolvedFilterFn<TData>,
        resolved_value: Value,
    }

    let column_by_id: HashMap<&str, &ColumnDef<TData>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    let mut resolved_column_filters: Vec<ResolvedColumnFilter<TData>> = Vec::new();
    for spec in column_filters {
        let Some(column) = column_by_id.get(spec.column.as_ref()).copied() else {
            continue;
        };
        let Some(f) = resolve_column_filter_fn(row_model, column, filter_fns) else {
            continue;
        };
        let resolved_value = f.resolve_filter_value(&spec.value);
        resolved_column_filters.push(ResolvedColumnFilter {
            column_id: spec.column.clone(),
            f,
            resolved_value,
        });
    }

    let mut resolved_global_filters: Vec<ResolvedGlobalFilter<TData>> = Vec::new();
    if let Some(global_filter_value) = global_filter.as_ref() {
        if options.enable_filters && options.enable_global_filter {
            let first_row_original = row_model
                .flat_rows()
                .first()
                .and_then(|&i| row_model.row(i))
                .map(|r| r.original);

            for col in columns {
                if !col.enable_global_filter {
                    continue;
                }

                let can_global_filter = match get_column_can_global_filter {
                    Some(f) => first_row_original.is_some_and(|first| f(col, first)),
                    None => match first_row_original {
                        Some(first) => match col.sort_value.as_ref() {
                            Some(get_value) => matches!(
                                (get_value)(first),
                                TanStackValue::String(_) | TanStackValue::Number(_)
                            ),
                            None => col.filter_fn.is_some() || col.filter_fn_with_meta.is_some(),
                        },
                        _ => false,
                    },
                };

                if !can_global_filter {
                    continue;
                }

                if let Some(value) = col.sort_value.as_ref().cloned() {
                    let resolved_global = match global_filter_fn {
                        FilteringFnSpec::Auto => ResolvedFilterFn::BuiltIn {
                            builtin: BuiltInFilterFn::IncludesString,
                            value,
                        },
                        FilteringFnSpec::BuiltIn(builtin) => ResolvedFilterFn::BuiltIn {
                            builtin: *builtin,
                            value,
                        },
                        FilteringFnSpec::Named(key) => match filter_fns.get(key) {
                            Some(FilterFnDef::BuiltIn(builtin)) => ResolvedFilterFn::BuiltIn {
                                builtin: *builtin,
                                value,
                            },
                            Some(FilterFnDef::Value(f)) => ResolvedFilterFn::Value {
                                value,
                                f: f.clone(),
                            },
                            Some(FilterFnDef::ValueWithMeta(f)) => {
                                ResolvedFilterFn::ValueWithMeta {
                                    value,
                                    f: f.clone(),
                                }
                            }
                            None => {
                                let builtin = builtin_filter_key(key.as_ref())
                                    .unwrap_or(BuiltInFilterFn::IncludesString);
                                ResolvedFilterFn::BuiltIn { builtin, value }
                            }
                        },
                    };

                    let resolved_value = resolved_global.resolve_filter_value(global_filter_value);
                    resolved_global_filters.push(ResolvedGlobalFilter {
                        column_id: col.id.clone(),
                        f: resolved_global,
                        resolved_value,
                    });
                    continue;
                }

                if let Some(custom) = col.filter_fn_with_meta.as_ref() {
                    let f = ResolvedFilterFn::CustomWithMeta(custom.clone());
                    let resolved_value = f.resolve_filter_value(global_filter_value);
                    resolved_global_filters.push(ResolvedGlobalFilter {
                        column_id: col.id.clone(),
                        f,
                        resolved_value,
                    });
                    continue;
                }

                if let Some(custom) = col.filter_fn.as_ref() {
                    let f = ResolvedFilterFn::Custom(custom.clone());
                    let resolved_value = f.resolve_filter_value(global_filter_value);
                    resolved_global_filters.push(ResolvedGlobalFilter {
                        column_id: col.id.clone(),
                        f,
                        resolved_value,
                    });
                    continue;
                }
            }
        }
    }

    let mut snapshot = RowFilterStateSnapshot {
        filterable_ids: resolved_column_filters
            .iter()
            .map(|spec| spec.column_id.clone())
            .collect(),
        row_column_filters: HashMap::new(),
        row_column_filters_meta: HashMap::new(),
    };

    let global_filter_id: ColumnId = Arc::from("__global__");
    if !resolved_global_filters.is_empty() {
        snapshot.filterable_ids.push(global_filter_id.clone());
    }

    for &row_index in row_model.flat_rows() {
        let Some(row) = row_model.row(row_index) else {
            continue;
        };

        let mut column_filters: RowColumnFilters = HashMap::new();
        let mut column_filters_meta: RowColumnFiltersMeta = HashMap::new();

        for spec in &resolved_column_filters {
            let mut add_meta = |meta: Value| {
                column_filters_meta.insert(spec.column_id.clone(), meta);
            };
            let pass = spec.f.apply_with_meta(
                row.original,
                &spec.column_id,
                &spec.resolved_value,
                &mut add_meta,
            );
            column_filters.insert(spec.column_id.clone(), pass);
        }

        if !resolved_global_filters.is_empty() {
            let mut global_pass = false;
            for spec in &resolved_global_filters {
                let mut add_meta = |meta: Value| {
                    column_filters_meta.insert(spec.column_id.clone(), meta);
                };
                if spec.f.apply_with_meta(
                    row.original,
                    &spec.column_id,
                    &spec.resolved_value,
                    &mut add_meta,
                ) {
                    global_pass = true;
                    break;
                }
            }
            column_filters.insert(global_filter_id.clone(), global_pass);
        }

        snapshot.row_column_filters.insert(row.key, column_filters);
        snapshot
            .row_column_filters_meta
            .insert(row.key, column_filters_meta);
    }

    snapshot
}

pub fn filter_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    column_filters: &[ColumnFilter],
    global_filter: GlobalFilterState,
    options: TableOptions,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    global_filter_fn: &FilteringFnSpec,
    get_column_can_global_filter: Option<&dyn Fn(&ColumnDef<TData>, &TData) -> bool>,
) -> RowModel<'a, TData> {
    if row_model.root_rows().is_empty() {
        return row_model.clone();
    }

    let snapshot = evaluate_row_filter_state(
        row_model,
        columns,
        column_filters,
        global_filter,
        options,
        filter_fns,
        global_filter_fn,
        get_column_can_global_filter,
    );

    if snapshot.filterable_ids.is_empty() {
        return row_model.clone();
    }

    fn row_passes(snapshot: &RowFilterStateSnapshot, row_key: RowKey) -> bool {
        let Some(row_filters) = snapshot.row_column_filters.get(&row_key) else {
            return true;
        };
        for filter_id in &snapshot.filterable_ids {
            if row_filters.get(filter_id).is_some_and(|pass| !*pass) {
                return false;
            }
        }
        true
    }

    fn push_row_shell<'a, TData>(
        source: &RowModel<'a, TData>,
        original: RowIndex,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Option<RowIndex> {
        let row = source.row(original)?;
        let new_index = out_arena.len();
        out_arena.push(super::Row {
            id: row.id.clone(),
            key: row.key,
            original: row.original,
            index: row.index,
            depth: row.depth,
            parent: None,
            parent_key: None,
            sub_rows: Vec::new(),
        });
        Some(new_index)
    }

    fn clone_subtree_unfiltered<'a, TData>(
        source: &RowModel<'a, TData>,
        original: RowIndex,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Option<RowIndex> {
        let row = source.row(original)?;
        let new_index = push_row_shell(source, original, out_arena)?;
        let mut child_new_indices: Vec<RowIndex> = Vec::new();
        for &child in &row.sub_rows {
            let Some(child_new) = clone_subtree_unfiltered(source, child, out_arena) else {
                continue;
            };
            child_new_indices.push(child_new);
        }

        for &child in &child_new_indices {
            if let Some(child_row) = out_arena.get_mut(child) {
                child_row.parent = Some(new_index);
                child_row.parent_key = Some(row.key);
            }
        }
        if let Some(new_row) = out_arena.get_mut(new_index) {
            new_row.sub_rows = child_new_indices;
        }

        Some(new_index)
    }

    fn register_included_row<'a, TData>(
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_rows_by_id: &mut HashMap<super::RowId, RowIndex>,
        out_arena: &[super::Row<'a, TData>],
        row_index: RowIndex,
    ) {
        out_flat_rows.push(row_index);
        if let Some(row) = out_arena.get(row_index) {
            out_rows_by_key.insert(row.key, row_index);
            out_rows_by_id.insert(row.id.clone(), row_index);
        }
    }

    fn recurse_filter_from_root<'a, TData>(
        source: &RowModel<'a, TData>,
        rows_to_filter: &[RowIndex],
        depth: usize,
        max_depth: usize,
        snapshot: &RowFilterStateSnapshot,
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_rows_by_id: &mut HashMap<super::RowId, RowIndex>,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Vec<RowIndex> {
        let mut out_rows: Vec<RowIndex> = Vec::new();

        for &original in rows_to_filter {
            let Some(row) = source.row(original) else {
                continue;
            };

            if !row_passes(snapshot, row.key) {
                continue;
            }

            let has_children = !row.sub_rows.is_empty();
            let new_index = if has_children && depth < max_depth {
                let child_rows = recurse_filter_from_root(
                    source,
                    row.sub_rows.as_slice(),
                    depth + 1,
                    max_depth,
                    snapshot,
                    out_flat_rows,
                    out_rows_by_key,
                    out_rows_by_id,
                    out_arena,
                );

                let Some(new_index) = push_row_shell(source, original, out_arena) else {
                    continue;
                };

                for &child in &child_rows {
                    if let Some(child_row) = out_arena.get_mut(child) {
                        child_row.parent = Some(new_index);
                        child_row.parent_key = Some(row.key);
                    }
                }
                if let Some(new_row) = out_arena.get_mut(new_index) {
                    new_row.sub_rows = child_rows;
                }

                new_index
            } else if has_children {
                let Some(new_index) = clone_subtree_unfiltered(source, original, out_arena) else {
                    continue;
                };
                new_index
            } else {
                let Some(new_index) = push_row_shell(source, original, out_arena) else {
                    continue;
                };
                new_index
            };

            out_rows.push(new_index);
            register_included_row(
                out_flat_rows,
                out_rows_by_key,
                out_rows_by_id,
                out_arena,
                new_index,
            );
        }

        out_rows
    }

    fn recurse_filter_from_leafs<'a, TData>(
        source: &RowModel<'a, TData>,
        rows_to_filter: &[RowIndex],
        depth: usize,
        max_depth: usize,
        snapshot: &RowFilterStateSnapshot,
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_rows_by_id: &mut HashMap<super::RowId, RowIndex>,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Vec<RowIndex> {
        let mut out_rows: Vec<RowIndex> = Vec::new();

        for &original in rows_to_filter {
            let Some(row) = source.row(original) else {
                continue;
            };

            let has_children = !row.sub_rows.is_empty();
            let included_children = if has_children && depth < max_depth {
                recurse_filter_from_leafs(
                    source,
                    row.sub_rows.as_slice(),
                    depth + 1,
                    max_depth,
                    snapshot,
                    out_flat_rows,
                    out_rows_by_key,
                    out_rows_by_id,
                    out_arena,
                )
            } else {
                Vec::new()
            };

            let self_passes = row_passes(snapshot, row.key);
            let should_include = if has_children && depth < max_depth {
                self_passes || !included_children.is_empty()
            } else {
                self_passes
            };

            if !should_include {
                continue;
            }

            let Some(new_index) = push_row_shell(source, original, out_arena) else {
                continue;
            };

            for &child in &included_children {
                if let Some(child_row) = out_arena.get_mut(child) {
                    child_row.parent = Some(new_index);
                    child_row.parent_key = Some(row.key);
                }
            }
            if let Some(new_row) = out_arena.get_mut(new_index) {
                new_row.sub_rows = included_children;
            }

            out_rows.push(new_index);
            register_included_row(
                out_flat_rows,
                out_rows_by_key,
                out_rows_by_id,
                out_arena,
                new_index,
            );
        }

        out_rows
    }

    let mut out_flat_rows: Vec<RowIndex> = Vec::new();
    let mut out_rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut out_rows_by_id: HashMap<super::RowId, RowIndex> = HashMap::new();
    let mut out_arena: Vec<super::Row<'a, TData>> = Vec::new();

    let max_depth = options.max_leaf_row_filter_depth;
    let out_root_rows = if options.filter_from_leaf_rows {
        recurse_filter_from_leafs(
            row_model,
            row_model.root_rows(),
            0,
            max_depth,
            &snapshot,
            &mut out_flat_rows,
            &mut out_rows_by_key,
            &mut out_rows_by_id,
            &mut out_arena,
        )
    } else {
        recurse_filter_from_root(
            row_model,
            row_model.root_rows(),
            0,
            max_depth,
            &snapshot,
            &mut out_flat_rows,
            &mut out_rows_by_key,
            &mut out_rows_by_id,
            &mut out_arena,
        )
    };

    RowModel {
        root_rows: out_root_rows,
        flat_rows: out_flat_rows,
        rows_by_key: out_rows_by_key,
        rows_by_id: out_rows_by_id,
        arena: out_arena,
    }
}
pub fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }

    let needle = needle.as_bytes();
    let hay = haystack.as_bytes();

    for start in 0..=hay.len().saturating_sub(needle.len()) {
        if needle
            .iter()
            .enumerate()
            .all(|(i, &b)| hay[start + i].to_ascii_lowercase() == b.to_ascii_lowercase())
        {
            return true;
        }
    }
    false
}

fn builtin_filter_key(key: &str) -> Option<BuiltInFilterFn> {
    match key {
        "includesString" => Some(BuiltInFilterFn::IncludesString),
        "includesStringSensitive" => Some(BuiltInFilterFn::IncludesStringSensitive),
        "equalsString" => Some(BuiltInFilterFn::EqualsString),
        "arrIncludes" => Some(BuiltInFilterFn::ArrIncludes),
        "arrIncludesAll" => Some(BuiltInFilterFn::ArrIncludesAll),
        "arrIncludesSome" => Some(BuiltInFilterFn::ArrIncludesSome),
        "equals" => Some(BuiltInFilterFn::Equals),
        "weakEquals" => Some(BuiltInFilterFn::WeakEquals),
        "inNumberRange" => Some(BuiltInFilterFn::InNumberRange),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolvedFilterBehavior {
    BuiltIn(BuiltInFilterFn),
    Custom,
}

fn resolve_filter_fn_for_column<TData>(
    data: &[TData],
    column: &ColumnDef<TData>,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
) -> Option<ResolvedFilterBehavior> {
    if column.filter_fn.is_some() || column.filter_fn_with_meta.is_some() {
        return Some(ResolvedFilterBehavior::Custom);
    }

    let spec = column.filtering_fn.as_ref()?;
    match spec {
        FilteringFnSpec::Auto => {
            let Some(first) = data.first() else {
                return Some(ResolvedFilterBehavior::BuiltIn(BuiltInFilterFn::WeakEquals));
            };
            let value = match column.sort_value.as_ref() {
                Some(get_value) => get_value(first),
                None => TanStackValue::Undefined,
            };
            let builtin = match value {
                TanStackValue::String(_) => BuiltInFilterFn::IncludesString,
                TanStackValue::Number(_) => BuiltInFilterFn::InNumberRange,
                TanStackValue::Bool(_) => BuiltInFilterFn::Equals,
                TanStackValue::DateTime(_) => BuiltInFilterFn::Equals,
                TanStackValue::Array(_) => BuiltInFilterFn::Equals,
                _ => BuiltInFilterFn::WeakEquals,
            };
            Some(ResolvedFilterBehavior::BuiltIn(builtin))
        }
        FilteringFnSpec::BuiltIn(builtin) => Some(ResolvedFilterBehavior::BuiltIn(*builtin)),
        FilteringFnSpec::Named(key) => match filter_fns.get(key) {
            Some(FilterFnDef::BuiltIn(builtin)) => Some(ResolvedFilterBehavior::BuiltIn(*builtin)),
            Some(FilterFnDef::Value(_) | FilterFnDef::ValueWithMeta(_)) => {
                Some(ResolvedFilterBehavior::Custom)
            }
            None => builtin_filter_key(key.as_ref()).map(ResolvedFilterBehavior::BuiltIn),
        },
    }
}

fn is_json_filter_value_empty(value: &Value) -> bool {
    value.is_null() || value.as_str().is_some_and(|s| s.is_empty())
}

fn is_json_falsey(value: &Value) -> bool {
    value.is_null() || value.as_str().is_some_and(|s| s.is_empty())
}

fn should_auto_remove_built_in_filter(builtin: BuiltInFilterFn, value: &Value) -> bool {
    if is_json_filter_value_empty(value) {
        return true;
    }

    match builtin {
        BuiltInFilterFn::ArrIncludesAll | BuiltInFilterFn::ArrIncludesSome => {
            value.as_array().is_some_and(|arr| arr.is_empty())
        }
        BuiltInFilterFn::InNumberRange => value.as_array().is_some_and(|arr| {
            arr.len() == 2 && is_json_falsey(&arr[0]) && is_json_falsey(&arr[1])
        }),
        _ => false,
    }
}

fn tanstack_value_to_string(v: &TanStackValue) -> Option<String> {
    match v {
        TanStackValue::Undefined | TanStackValue::Null => None,
        TanStackValue::Bool(b) => Some(if *b { "true" } else { "false" }.to_string()),
        TanStackValue::Number(n) => Some(n.to_string()),
        TanStackValue::String(s) => Some(s.as_ref().to_string()),
        TanStackValue::DateTime(ms) => Some(ms.to_string()),
        TanStackValue::Array(items) => Some(
            items
                .iter()
                .filter_map(tanstack_value_to_string)
                .collect::<Vec<_>>()
                .join(","),
        ),
    }
}

fn tanstack_value_strict_eq_json(cell: &TanStackValue, filter: &Value) -> bool {
    match (cell, filter) {
        (TanStackValue::Undefined, Value::Null) => false,
        (TanStackValue::Null, Value::Null) => true,
        (TanStackValue::Bool(a), Value::Bool(b)) => a == b,
        (TanStackValue::Number(a), Value::Number(b)) => b.as_f64().is_some_and(|bf| bf == *a),
        (TanStackValue::String(a), Value::String(b)) => a.as_ref() == b,
        _ => false,
    }
}

fn json_to_number(v: &Value) -> f64 {
    match v {
        Value::Null => 0.0,
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.trim().parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    }
}

fn tanstack_value_weak_eq_json(cell: &TanStackValue, filter: &Value) -> bool {
    if tanstack_value_strict_eq_json(cell, filter) {
        return true;
    }

    match cell {
        TanStackValue::Number(a) => {
            let b = json_to_number(filter);
            !b.is_nan() && *a == b
        }
        TanStackValue::String(a) => {
            if let Some(b) = filter.as_str() {
                return a.as_ref() == b;
            }
            let a_num = a.as_ref().trim().parse::<f64>().unwrap_or(f64::NAN);
            let b_num = json_to_number(filter);
            !a_num.is_nan() && !b_num.is_nan() && a_num == b_num
        }
        TanStackValue::Bool(a) => {
            let a_num = if *a { 1.0 } else { 0.0 };
            let b_num = json_to_number(filter);
            !b_num.is_nan() && a_num == b_num
        }
        TanStackValue::Null => filter.as_str().is_some_and(|s| s.is_empty()) || filter.is_null(),
        TanStackValue::Undefined => false,
        _ => false,
    }
}

fn apply_built_in_filter_fn(
    builtin: BuiltInFilterFn,
    cell: &TanStackValue,
    filter: &Value,
) -> bool {
    match builtin {
        BuiltInFilterFn::IncludesString => {
            let Some(search) = filter.as_str() else {
                return false;
            };
            let search = search.to_string().to_lowercase();
            let Some(hay) = tanstack_value_to_string(cell) else {
                return false;
            };
            hay.to_lowercase().contains(&search)
        }
        BuiltInFilterFn::IncludesStringSensitive => {
            let Some(search) = filter.as_str() else {
                return false;
            };
            let Some(hay) = tanstack_value_to_string(cell) else {
                return false;
            };
            hay.contains(search)
        }
        BuiltInFilterFn::EqualsString => {
            let Some(search) = filter.as_str() else {
                return false;
            };
            let Some(hay) = tanstack_value_to_string(cell) else {
                return false;
            };
            hay.to_lowercase() == search.to_lowercase()
        }
        BuiltInFilterFn::ArrIncludes => match cell {
            TanStackValue::Array(items) => items
                .iter()
                .any(|it| tanstack_value_strict_eq_json(it, filter)),
            _ => false,
        },
        BuiltInFilterFn::ArrIncludesAll => {
            let Some(want) = filter.as_array() else {
                return false;
            };
            let TanStackValue::Array(items) = cell else {
                return false;
            };
            !want.iter().any(|needle| {
                !items
                    .iter()
                    .any(|it| tanstack_value_strict_eq_json(it, needle))
            })
        }
        BuiltInFilterFn::ArrIncludesSome => {
            let Some(want) = filter.as_array() else {
                return false;
            };
            let TanStackValue::Array(items) = cell else {
                return false;
            };
            want.iter().any(|needle| {
                items
                    .iter()
                    .any(|it| tanstack_value_strict_eq_json(it, needle))
            })
        }
        BuiltInFilterFn::Equals => tanstack_value_strict_eq_json(cell, filter),
        BuiltInFilterFn::WeakEquals => tanstack_value_weak_eq_json(cell, filter),
        BuiltInFilterFn::InNumberRange => {
            let Some(range) = filter.as_array() else {
                return false;
            };
            if range.len() != 2 {
                return false;
            }
            let TanStackValue::Number(row_value) = cell else {
                return false;
            };
            let min = json_to_number(&range[0]);
            let max = json_to_number(&range[1]);
            if min.is_nan() || max.is_nan() {
                return false;
            }
            *row_value >= min && *row_value <= max
        }
    }
}

fn resolve_in_number_range(value: &Value) -> Value {
    let Some(range) = value.as_array() else {
        return value.clone();
    };
    if range.len() != 2 {
        return value.clone();
    }

    let unsafe_min = &range[0];
    let unsafe_max = &range[1];

    let parsed_min = match unsafe_min {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    };
    let parsed_max = match unsafe_max {
        Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
        Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN),
        _ => f64::NAN,
    };

    let mut min = if unsafe_min.is_null() || parsed_min.is_nan() {
        f64::NEG_INFINITY
    } else {
        parsed_min
    };
    let mut max = if unsafe_max.is_null() || parsed_max.is_nan() {
        f64::INFINITY
    } else {
        parsed_max
    };

    if min > max {
        std::mem::swap(&mut min, &mut max);
    }

    Value::Array(vec![Value::from(min), Value::from(max)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{Table, create_column_helper};

    #[derive(Debug, Clone)]
    struct Item {
        name: Arc<str>,
        role: Arc<str>,
    }

    #[test]
    fn filter_row_model_applies_column_filters_and_keeps_stable_keys() {
        let data = vec![
            Item {
                name: "a".into(),
                role: "Admin".into(),
            },
            Item {
                name: "b".into(),
                role: "Member".into(),
            },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper
                .clone()
                .accessor("name", |it| it.name.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.name.as_ref(), q)),
            helper
                .accessor("role", |it| it.role.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.role.as_ref(), q)),
        ];

        let table = Table::builder(&data).columns(columns).build();
        let core = table.core_row_model();

        let filtered = filter_row_model(
            core,
            table.columns(),
            &[ColumnFilter {
                column: "role".into(),
                value: Value::from("Admin"),
            }],
            None,
            TableOptions::default(),
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
        );

        assert_eq!(filtered.root_rows().len(), 1);
        let row = filtered.row(filtered.root_rows()[0]).expect("row");
        assert_eq!(row.key, crate::table::RowKey::from_index(0));
        assert!(filtered.row_by_key(row.key).is_some());
    }

    #[derive(Debug, Clone)]
    struct TreeNode {
        id: u64,
        label: Arc<str>,
        children: Vec<TreeNode>,
    }

    fn tree_table(data: Vec<TreeNode>) -> Table<'static, TreeNode> {
        let data: &'static [TreeNode] = Box::leak(data.into_boxed_slice());
        let helper = create_column_helper::<TreeNode>();
        let columns = vec![
            helper
                .accessor("label", |it| it.label.clone())
                .filter_by(|row, q| row.label.as_ref() == q),
        ];

        Table::builder(data)
            .columns(columns)
            .get_row_key(|row, _idx, _parent| RowKey(row.id))
            .get_row_id(|row, _idx, _parent| super::super::RowId::new(row.id.to_string()))
            .get_sub_rows(|row, _idx| Some(row.children.as_slice()))
            .build()
    }

    fn root_keys(model: &RowModel<'_, TreeNode>) -> Vec<u64> {
        model
            .root_rows()
            .iter()
            .filter_map(|&i| model.row(i).map(|r| r.key.0))
            .collect()
    }

    fn flat_keys(model: &RowModel<'_, TreeNode>) -> Vec<u64> {
        model
            .flat_rows()
            .iter()
            .filter_map(|&i| model.row(i).map(|r| r.key.0))
            .collect()
    }

    #[test]
    fn root_filter_depth_zero_preserves_unfiltered_subtree() {
        let table = tree_table(vec![
            TreeNode {
                id: 1,
                label: Arc::from("match"),
                children: vec![
                    TreeNode {
                        id: 11,
                        label: Arc::from("miss"),
                        children: Vec::new(),
                    },
                    TreeNode {
                        id: 12,
                        label: Arc::from("match"),
                        children: Vec::new(),
                    },
                ],
            },
            TreeNode {
                id: 2,
                label: Arc::from("miss"),
                children: vec![TreeNode {
                    id: 21,
                    label: Arc::from("match"),
                    children: Vec::new(),
                }],
            },
        ]);

        let filtered = filter_row_model(
            table.core_row_model(),
            table.columns(),
            &[ColumnFilter {
                column: "label".into(),
                value: Value::String("match".to_string()),
            }],
            None,
            TableOptions {
                max_leaf_row_filter_depth: 0,
                ..TableOptions::default()
            },
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
        );

        assert_eq!(root_keys(&filtered), vec![1]);
        assert_eq!(flat_keys(&filtered), vec![1]);

        let root_index = filtered.row_by_key(RowKey(1)).expect("root row");
        let root_row = filtered.row(root_index).expect("root row exists");
        let child_keys: Vec<u64> = root_row
            .sub_rows
            .iter()
            .filter_map(|&i| filtered.row(i).map(|r| r.key.0))
            .collect();
        assert_eq!(child_keys, vec![11, 12]);
    }

    #[test]
    fn leaf_filter_depth_gate_controls_descendant_bubbling() {
        let table = tree_table(vec![
            TreeNode {
                id: 1,
                label: Arc::from("miss"),
                children: vec![TreeNode {
                    id: 11,
                    label: Arc::from("miss"),
                    children: vec![TreeNode {
                        id: 111,
                        label: Arc::from("match"),
                        children: Vec::new(),
                    }],
                }],
            },
            TreeNode {
                id: 2,
                label: Arc::from("match"),
                children: Vec::new(),
            },
        ]);

        let filtered_depth_1 = filter_row_model(
            table.core_row_model(),
            table.columns(),
            &[ColumnFilter {
                column: "label".into(),
                value: Value::String("match".to_string()),
            }],
            None,
            TableOptions {
                filter_from_leaf_rows: true,
                max_leaf_row_filter_depth: 1,
                ..TableOptions::default()
            },
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
        );

        assert_eq!(root_keys(&filtered_depth_1), vec![2]);
        assert_eq!(flat_keys(&filtered_depth_1), vec![2]);

        let filtered_depth_2 = filter_row_model(
            table.core_row_model(),
            table.columns(),
            &[ColumnFilter {
                column: "label".into(),
                value: Value::String("match".to_string()),
            }],
            None,
            TableOptions {
                filter_from_leaf_rows: true,
                max_leaf_row_filter_depth: 2,
                ..TableOptions::default()
            },
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
        );

        assert_eq!(root_keys(&filtered_depth_2), vec![1, 2]);
        assert_eq!(flat_keys(&filtered_depth_2), vec![111, 11, 1, 2]);
    }

    #[test]
    fn evaluate_row_filter_state_tracks_per_row_pass_map() {
        let data = vec![
            Item {
                name: "a".into(),
                role: "Admin".into(),
            },
            Item {
                name: "b".into(),
                role: "Member".into(),
            },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper
                .clone()
                .accessor("name", |it| it.name.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.name.as_ref(), q)),
            helper
                .accessor("role", |it| it.role.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.role.as_ref(), q)),
        ];

        let table = Table::builder(&data).columns(columns).build();
        let snapshot = evaluate_row_filter_state(
            table.core_row_model(),
            table.columns(),
            &[ColumnFilter {
                column: "role".into(),
                value: Value::String("Admin".to_string()),
            }],
            Some(Value::String("Admin".to_string())),
            TableOptions::default(),
            &HashMap::new(),
            &FilteringFnSpec::Auto,
            None,
        );

        assert_eq!(
            snapshot.filterable_ids,
            vec![Arc::from("role"), Arc::from("__global__")]
        );

        let row0 = snapshot
            .row_column_filters
            .get(&RowKey::from_index(0))
            .expect("row 0 filters");
        assert_eq!(row0.get("role"), Some(&true));
        assert_eq!(row0.get("__global__"), Some(&true));

        let row1 = snapshot
            .row_column_filters
            .get(&RowKey::from_index(1))
            .expect("row 1 filters");
        assert_eq!(row1.get("role"), Some(&false));
        assert_eq!(row1.get("__global__"), Some(&false));

        assert!(
            snapshot
                .row_column_filters_meta
                .get(&RowKey::from_index(0))
                .is_some_and(|meta| meta.is_empty())
        );
        assert!(
            snapshot
                .row_column_filters_meta
                .get(&RowKey::from_index(1))
                .is_some_and(|meta| meta.is_empty())
        );
    }
}
