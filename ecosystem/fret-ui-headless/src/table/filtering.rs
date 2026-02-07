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
    if column_filters.is_empty() && global_filter.is_none() {
        return row_model.clone();
    }

    #[derive(Clone)]
    enum ResolvedFilterFn<TData> {
        Custom(super::FilterFn<TData>),
        BuiltIn {
            builtin: BuiltInFilterFn,
            value: SortValueFn<TData>,
        },
        Value {
            value: SortValueFn<TData>,
            f: Arc<dyn Fn(&TanStackValue, &Value) -> bool>,
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

        fn apply(&self, row: &TData, _column_id: &ColumnId, filter_value: &Value) -> bool {
            match self {
                Self::Custom(f) => f(row, filter_value),
                Self::BuiltIn { builtin, value } => {
                    let v = value(row);
                    apply_built_in_filter_fn(*builtin, &v, filter_value)
                }
                Self::Value { value, f } => {
                    let v = value(row);
                    f(&v, filter_value)
                }
            }
        }
    }

    fn resolve_column_filter_fn<TData>(
        row_model: &RowModel<'_, TData>,
        column: &ColumnDef<TData>,
        filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    ) -> Option<ResolvedFilterFn<TData>> {
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
                    // Upstream order: `typeof value === 'object'` check happens before `Array.isArray`.
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

    #[derive(Clone)]
    struct ResolvedGlobalFilter<TData> {
        column_id: ColumnId,
        f: ResolvedFilterFn<TData>,
        resolved_value: Value,
    }

    let mut resolved_global_filters: Vec<ResolvedGlobalFilter<TData>> = Vec::new();
    if let Some(global_filter_value) = global_filter.as_ref() {
        // TanStack: global filters only apply to columns that can global filter.
        // Column filters (columnFilters state) do not consult `getCanFilter()` at row-model time.
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
                let global_value = global_filter_value.clone();

                let can_global_filter = match (get_column_can_global_filter, first_row_original) {
                    (Some(hook), Some(first)) => hook(col, first),
                    (None, Some(first)) => {
                        match col.sort_value.as_ref() {
                            Some(get_value) => matches!(
                                (get_value)(first),
                                TanStackValue::String(_) | TanStackValue::Number(_)
                            ),
                            None => {
                                // Fallback for v0 consumers: if the column provides an explicit
                                // filter predicate, allow global filtering to reuse it.
                                col.filter_fn.is_some()
                            }
                        }
                    }
                    _ => false,
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
                            None => {
                                let builtin = builtin_filter_key(key.as_ref())
                                    .unwrap_or(BuiltInFilterFn::IncludesString);
                                ResolvedFilterFn::BuiltIn { builtin, value }
                            }
                        },
                    };

                    let resolved_value = resolved_global.resolve_filter_value(&global_value);
                    resolved_global_filters.push(ResolvedGlobalFilter {
                        column_id: col.id.clone(),
                        f: resolved_global,
                        resolved_value,
                    });
                    continue;
                }

                // Fallback: if the caller provided a string-based column filter function, reuse it
                // for global filtering when there is no `getValue()` extractor.
                if let Some(custom) = col.filter_fn.as_ref() {
                    let f = ResolvedFilterFn::Custom(custom.clone());
                    let resolved_value = f.resolve_filter_value(&global_value);
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

    fn matches_column_filters<TData>(
        row: &super::Row<'_, TData>,
        filters: &[ResolvedColumnFilter<TData>],
    ) -> bool {
        for spec in filters {
            if !spec
                .f
                .apply(row.original, &spec.column_id, &spec.resolved_value)
            {
                return false;
            }
        }
        true
    }

    fn matches_global_filter<TData>(
        row: &super::Row<'_, TData>,
        filters: &[ResolvedGlobalFilter<TData>],
    ) -> bool {
        for spec in filters {
            if spec
                .f
                .apply(row.original, &spec.column_id, &spec.resolved_value)
            {
                return true;
            }
        }
        false
    }

    fn include_row<TData>(
        row: &super::Row<'_, TData>,
        column_filters: &[ResolvedColumnFilter<TData>],
        global_filters: &[ResolvedGlobalFilter<TData>],
    ) -> bool {
        if !matches_column_filters(row, column_filters) {
            return false;
        }
        if global_filters.is_empty() {
            return true;
        }
        matches_global_filter(row, global_filters)
    }

    let mut out_root_rows: Vec<RowIndex> = Vec::new();
    let mut out_flat_rows: Vec<RowIndex> = Vec::new();
    let mut out_rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut out_rows_by_id: HashMap<super::RowId, RowIndex> = HashMap::new();
    let mut out_arena: Vec<super::Row<'a, TData>> = Vec::new();

    fn recurse<'a, TData>(
        source: &RowModel<'a, TData>,
        column_filters: &[ResolvedColumnFilter<TData>],
        global_filters: &[ResolvedGlobalFilter<TData>],
        original: RowIndex,
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_rows_by_id: &mut HashMap<super::RowId, RowIndex>,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Option<RowIndex> {
        let row = source.row(original)?;

        let mut included_children: Vec<RowIndex> = Vec::new();
        for child in &row.sub_rows {
            if let Some(child_new) = recurse(
                source,
                column_filters,
                global_filters,
                *child,
                out_flat_rows,
                out_rows_by_key,
                out_rows_by_id,
                out_arena,
            ) {
                included_children.push(child_new);
            }
        }

        let self_matches = include_row(row, column_filters, global_filters);
        let should_include = self_matches || !included_children.is_empty();
        if !should_include {
            return None;
        }

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
        out_flat_rows.push(new_index);
        out_rows_by_key.insert(row.key, new_index);
        out_rows_by_id.insert(row.id.clone(), new_index);

        for &child in &included_children {
            if let Some(child_row) = out_arena.get_mut(child) {
                child_row.parent = Some(new_index);
                child_row.parent_key = Some(row.key);
            }
        }
        if let Some(new_row) = out_arena.get_mut(new_index) {
            new_row.sub_rows = included_children;
        }

        Some(new_index)
    }

    for &root in row_model.root_rows() {
        if let Some(new_root) = recurse(
            row_model,
            &resolved_column_filters,
            &resolved_global_filters,
            root,
            &mut out_flat_rows,
            &mut out_rows_by_key,
            &mut out_rows_by_id,
            &mut out_arena,
        ) {
            out_root_rows.push(new_root);
        }
    }

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
    if column.filter_fn.is_some() {
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
            Some(FilterFnDef::Value(_)) => Some(ResolvedFilterBehavior::Custom),
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
}
