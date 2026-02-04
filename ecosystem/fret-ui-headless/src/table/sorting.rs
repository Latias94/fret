use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use super::{
    ColumnDef, ColumnId, RowIndex, RowKey, RowModel, SortCmpFn, SortUndefined, TableOptions,
    column::{BuiltInSortingFn, SortIsUndefinedFn, SortValueFn, SortingFnSpec, TanStackValue},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    pub column: ColumnId,
    pub desc: bool,
}

pub type SortingState = Vec<SortSpec>;

#[derive(Clone)]
pub enum SortingFnDef<TData> {
    BuiltIn(BuiltInSortingFn),
    Cmp(SortCmpFn<TData>),
}

pub fn sort_for_column(sorting: &[SortSpec], id: &str) -> Option<bool> {
    sorting
        .iter()
        .find(|s| s.column.as_ref() == id)
        .map(|s| s.desc)
}

pub fn toggle_sort_for_column(sorting: &mut SortingState, column: ColumnId, multi: bool) {
    let pos = sorting
        .iter()
        .position(|s| s.column.as_ref() == column.as_ref());

    let next = match pos.and_then(|i| sorting.get(i).map(|s| s.desc)) {
        None => Some(false),
        Some(false) => Some(true),
        Some(true) => None,
    };

    if !multi {
        sorting.clear();
        if let Some(desc) = next {
            sorting.push(SortSpec { column, desc });
        }
        return;
    }

    match (pos, next) {
        (None, Some(desc)) => sorting.push(SortSpec { column, desc }),
        (Some(i), Some(desc)) => {
            if let Some(spec) = sorting.get_mut(i) {
                spec.desc = desc;
            }
        }
        (Some(i), None) => {
            sorting.remove(i);
        }
        (None, None) => {}
    }
}

/// TanStack-aligned sorting state transition for `column.toggleSorting(undefined, multi)`.
///
/// This implements the upstream action logic (add/toggle/remove/replace) including:
/// - `sortDescFirst` (table + column)
/// - `enableSortingRemoval` (note: TanStack’s `toggleSorting` does not pass the `multi` flag to
///   `getNextSortingOrder`, so `enableMultiRemove` does not affect the direct toggle behavior)
/// - `enableMultiSort` / `maxMultiSortColCount`
///
/// Notes:
/// - This models the behavior of calling `column.toggleSorting` directly. It does not include UI
///   gating like `getCanSort()` (which TanStack applies in `getToggleSortingHandler`).
/// - This currently does not implement TanStack’s `getAutoSortDir()` inference (string => asc, else
///   desc) when `sortDescFirst` is unset. Until we add a stable “sort value extractor” surface,
///   the fallback first direction is `asc`.
pub fn toggle_sorting_tanstack<TData>(
    sorting: &mut SortingState,
    column: &ColumnDef<TData>,
    options: TableOptions,
    multi: bool,
    auto_sort_dir_desc: bool,
) {
    let existing_index = sorting
        .iter()
        .position(|s| s.column.as_ref() == column.id.as_ref());
    let existing_sorting = existing_index.and_then(|i| sorting.get(i)).cloned();

    // TanStack `column.getFirstSortDir()`:
    //
    // `sortDescFirst = columnDef.sortDescFirst ?? table.options.sortDescFirst ?? (getAutoSortDir() === 'desc')`
    let first_sort_direction_desc = column
        .sort_desc_first
        .or(options.sort_desc_first)
        .unwrap_or(auto_sort_dir_desc);

    let is_sorted = existing_sorting
        .as_ref()
        .map(|s| if s.desc { "desc" } else { "asc" });

    let enable_sorting_removal = options.enable_sorting_removal;

    let next_sorting_order = match is_sorted {
        None => Some(if first_sort_direction_desc {
            "desc"
        } else {
            "asc"
        }),
        Some(current) => {
            let first = if first_sort_direction_desc {
                "desc"
            } else {
                "asc"
            };
            if current != first && enable_sorting_removal {
                None
            } else if current == "desc" {
                Some("asc")
            } else {
                Some("desc")
            }
        }
    };

    let next_desc = next_sorting_order == Some("desc");

    let can_multi_sort = options.enable_multi_sort && column.enable_multi_sort;
    let multi = multi && can_multi_sort;

    enum SortAction {
        Add,
        Remove,
        Toggle,
        Replace,
    }

    let old_len = sorting.len();

    let mut sort_action = if old_len > 0 && multi {
        if existing_sorting.is_some() {
            SortAction::Toggle
        } else {
            SortAction::Add
        }
    } else {
        if old_len > 0 && existing_index.is_some_and(|i| i != old_len.saturating_sub(1)) {
            SortAction::Replace
        } else if existing_sorting.is_some() {
            SortAction::Toggle
        } else {
            SortAction::Replace
        }
    };

    if matches!(sort_action, SortAction::Toggle) && next_sorting_order.is_none() {
        sort_action = SortAction::Remove;
    }

    match sort_action {
        SortAction::Add => {
            sorting.push(SortSpec {
                column: column.id.clone(),
                desc: next_desc,
            });

            if let Some(max) = options.max_multi_sort_col_count {
                if max > 0 && sorting.len() > max {
                    let to_remove = sorting.len().saturating_sub(max);
                    sorting.drain(0..to_remove);
                }
            }
        }
        SortAction::Toggle => {
            if let Some(i) = existing_index {
                if let Some(spec) = sorting.get_mut(i) {
                    spec.desc = next_desc;
                }
            }
        }
        SortAction::Remove => {
            if let Some(i) = existing_index {
                sorting.remove(i);
            }
        }
        SortAction::Replace => {
            sorting.clear();
            sorting.push(SortSpec {
                column: column.id.clone(),
                desc: next_desc,
            });
        }
    }
}

/// TanStack-aligned sorting handler state transition (UI handler semantics).
///
/// This mirrors `column.getToggleSortingHandler()` which first checks `getCanSort()` (and thus
/// respects `enableSorting`) and uses `getCanMultiSort()` + `isMultiSortEvent` to decide whether a
/// multi-sort toggle is active.
pub fn toggle_sorting_handler_tanstack<TData>(
    sorting: &mut SortingState,
    column: &ColumnDef<TData>,
    options: TableOptions,
    event_multi: bool,
    auto_sort_dir_desc: bool,
) {
    let can_sort = options.enable_sorting
        && column.enable_sorting
        && (column.sort_cmp.is_some() || column.sort_value.is_some());
    if !can_sort {
        return;
    }

    let can_multi_sort = options.enable_multi_sort && column.enable_multi_sort;
    let multi = if can_multi_sort { event_multi } else { false };

    toggle_sorting_tanstack(sorting, column, options, multi, auto_sort_dir_desc);
}

fn builtin_sorting_fn_key(key: &str) -> Option<BuiltInSortingFn> {
    Some(match key {
        "alphanumeric" => BuiltInSortingFn::Alphanumeric,
        "alphanumericCaseSensitive" => BuiltInSortingFn::AlphanumericCaseSensitive,
        "text" => BuiltInSortingFn::Text,
        "textCaseSensitive" => BuiltInSortingFn::TextCaseSensitive,
        "datetime" => BuiltInSortingFn::Datetime,
        "basic" => BuiltInSortingFn::Basic,
        _ => return None,
    })
}

fn strict_equal(a: &TanStackValue, b: &TanStackValue) -> bool {
    match (a, b) {
        (TanStackValue::Undefined, TanStackValue::Undefined) => true,
        (TanStackValue::Null, TanStackValue::Null) => true,
        (TanStackValue::Bool(a), TanStackValue::Bool(b)) => a == b,
        (TanStackValue::Number(a), TanStackValue::Number(b)) => {
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a == b
            }
        }
        (TanStackValue::String(a), TanStackValue::String(b)) => a.as_ref() == b.as_ref(),
        (TanStackValue::DateTime(a), TanStackValue::DateTime(b)) => {
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a == b
            }
        }
        _ => false,
    }
}

fn to_number_for_compare(value: &TanStackValue) -> f64 {
    match value {
        TanStackValue::Undefined => f64::NAN,
        TanStackValue::Null => 0.0,
        TanStackValue::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        TanStackValue::Number(v) => *v,
        TanStackValue::DateTime(v) => *v,
        TanStackValue::String(s) => {
            let trimmed = s.as_ref().trim();
            if trimmed.is_empty() {
                0.0
            } else {
                trimmed.parse::<f64>().unwrap_or(f64::NAN)
            }
        }
        TanStackValue::Array(_) => f64::NAN,
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

fn abstract_lt(a: &TanStackValue, b: &TanStackValue) -> bool {
    match (a, b) {
        (TanStackValue::String(a), TanStackValue::String(b)) => a.as_ref() < b.as_ref(),
        _ => {
            let a = to_number_for_compare(a);
            let b = to_number_for_compare(b);
            if a.is_nan() || b.is_nan() {
                false
            } else {
                a < b
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

fn to_string_for_sorting(value: &TanStackValue) -> String {
    match value {
        TanStackValue::Number(v) => {
            if v.is_nan() || v.is_infinite() {
                String::new()
            } else {
                v.to_string()
            }
        }
        TanStackValue::String(s) => s.as_ref().to_string(),
        _ => String::new(),
    }
}

fn split_alpha_numeric_tokens(s: &str) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    let mut start: Option<usize> = None;
    let mut current_is_digit: Option<bool> = None;

    for (i, ch) in s.char_indices() {
        let is_digit = ch.is_ascii_digit();
        match (start, current_is_digit) {
            (None, None) => {
                start = Some(i);
                current_is_digit = Some(is_digit);
            }
            (Some(_), Some(kind)) if kind == is_digit => {}
            (Some(start_idx), Some(_)) => {
                if start_idx != i {
                    out.push(&s[start_idx..i]);
                }
                start = Some(i);
                current_is_digit = Some(is_digit);
            }
            _ => {}
        }
    }

    if let (Some(start_idx), Some(_)) = (start, current_is_digit) {
        if start_idx < s.len() {
            out.push(&s[start_idx..]);
        }
    }

    out.into_iter().filter(|seg| !seg.is_empty()).collect()
}

fn compare_alphanumeric(a_str: &str, b_str: &str) -> Ordering {
    let a = split_alpha_numeric_tokens(a_str);
    let b = split_alpha_numeric_tokens(b_str);

    let mut ai = 0usize;
    let mut bi = 0usize;

    while ai < a.len() && bi < b.len() {
        let aa = a[ai];
        let bb = b[bi];
        ai += 1;
        bi += 1;

        let an = aa.parse::<i64>().ok();
        let bn = bb.parse::<i64>().ok();

        match (an, bn) {
            (None, None) => {
                if aa > bb {
                    return Ordering::Greater;
                }
                if bb > aa {
                    return Ordering::Less;
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (Some(an), Some(bn)) => {
                if an > bn {
                    return Ordering::Greater;
                }
                if bn > an {
                    return Ordering::Less;
                }
            }
        }
    }

    a.len().cmp(&b.len())
}

fn auto_sorting_fn_builtin<'a, TData>(
    row_model: &RowModel<'a, TData>,
    column: &ColumnDef<TData>,
) -> BuiltInSortingFn {
    let Some(get_value) = column.sort_value.as_ref() else {
        return BuiltInSortingFn::Basic;
    };

    // TanStack: `table.getFilteredRowModel().flatRows.slice(10)` (note: skip the first 10 rows).
    let mut saw_string = false;
    for &row_index in row_model.flat_rows().iter().skip(10) {
        let Some(row) = row_model.row(row_index) else {
            continue;
        };
        let value = (get_value)(row.original);

        if matches!(value, TanStackValue::DateTime(_)) {
            return BuiltInSortingFn::Datetime;
        }

        if let TanStackValue::String(s) = value {
            saw_string = true;
            if split_alpha_numeric_tokens(s.as_ref()).len() > 1 {
                return BuiltInSortingFn::Alphanumeric;
            }
        }
    }

    if saw_string {
        BuiltInSortingFn::Text
    } else {
        BuiltInSortingFn::Basic
    }
}

fn compare_builtin_sort<TData>(
    kind: BuiltInSortingFn,
    get_value: &SortValueFn<TData>,
    a: &TData,
    b: &TData,
) -> Ordering {
    let a = (get_value)(a);
    let b = (get_value)(b);

    match kind {
        BuiltInSortingFn::Basic => compare_basic_tanstack(&a, &b),
        BuiltInSortingFn::Datetime => {
            if abstract_gt(&a, &b) {
                Ordering::Greater
            } else if abstract_lt(&a, &b) {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
        BuiltInSortingFn::Text => {
            let a = to_string_for_sorting(&a).to_lowercase();
            let b = to_string_for_sorting(&b).to_lowercase();
            if a == b {
                Ordering::Equal
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        BuiltInSortingFn::TextCaseSensitive => {
            let a = to_string_for_sorting(&a);
            let b = to_string_for_sorting(&b);
            if a == b {
                Ordering::Equal
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        BuiltInSortingFn::Alphanumeric => {
            let a = to_string_for_sorting(&a).to_lowercase();
            let b = to_string_for_sorting(&b).to_lowercase();
            compare_alphanumeric(&a, &b)
        }
        BuiltInSortingFn::AlphanumericCaseSensitive => {
            let a = to_string_for_sorting(&a);
            let b = to_string_for_sorting(&b);
            compare_alphanumeric(&a, &b)
        }
    }
}

#[derive(Clone)]
enum ResolvedSortCmp<TData> {
    Cmp(SortCmpFn<TData>),
    BuiltIn {
        kind: BuiltInSortingFn,
        get_value: SortValueFn<TData>,
    },
}

#[derive(Clone)]
enum ResolvedIsUndefined<TData> {
    Fn(SortIsUndefinedFn<TData>),
    Value(SortValueFn<TData>),
}

fn resolve_sort_cmp_for_column<'a, TData>(
    row_model: &RowModel<'a, TData>,
    column: &ColumnDef<TData>,
    sorting_fns: &HashMap<Arc<str>, SortingFnDef<TData>>,
) -> Option<ResolvedSortCmp<TData>> {
    if let Some(cmp) = column.sort_cmp.as_ref() {
        return Some(ResolvedSortCmp::Cmp(cmp.clone()));
    }

    let Some(spec) = column.sorting_fn.as_ref() else {
        return None;
    };

    match spec {
        SortingFnSpec::Auto => {
            let get_value = column.sort_value.as_ref()?.clone();
            let builtin = auto_sorting_fn_builtin(row_model, column);
            Some(ResolvedSortCmp::BuiltIn {
                kind: builtin,
                get_value,
            })
        }
        SortingFnSpec::BuiltIn(builtin) => {
            let get_value = column.sort_value.as_ref()?.clone();
            Some(ResolvedSortCmp::BuiltIn {
                kind: *builtin,
                get_value,
            })
        }
        SortingFnSpec::Named(key) => {
            if let Some(def) = sorting_fns.get(key.as_ref()) {
                return Some(match def {
                    SortingFnDef::Cmp(cmp) => ResolvedSortCmp::Cmp(cmp.clone()),
                    SortingFnDef::BuiltIn(builtin) => ResolvedSortCmp::BuiltIn {
                        kind: *builtin,
                        get_value: column.sort_value.as_ref()?.clone(),
                    },
                });
            }

            let builtin = builtin_sorting_fn_key(key.as_ref())?;
            let get_value = column.sort_value.as_ref()?.clone();
            Some(ResolvedSortCmp::BuiltIn {
                kind: builtin,
                get_value,
            })
        }
    }
}

pub fn sort_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    sorting: &[SortSpec],
    sorting_fns: &HashMap<Arc<str>, SortingFnDef<TData>>,
) -> RowModel<'a, TData> {
    if sorting.is_empty() || row_model.root_rows().is_empty() {
        return row_model.clone();
    }

    #[derive(Clone)]
    struct ResolvedSortColumn<TData> {
        cmp: ResolvedSortCmp<TData>,
        invert_sorting: bool,
        sort_undefined: Option<SortUndefined>,
        sort_is_undefined: Option<ResolvedIsUndefined<TData>>,
    }

    let sort_cfg_by_id: HashMap<&str, ResolvedSortColumn<TData>> = columns
        .iter()
        .filter_map(|c| {
            let cmp = resolve_sort_cmp_for_column(row_model, c, sorting_fns)?;

            // TanStack default: `sortUndefined: 1` (undefined values are last).
            //
            // We can only emulate this default when the column provides a TanStack-like
            // `getValue` surface (`sort_value_by`), so we can detect `undefined`.
            let sort_undefined = c.sort_undefined.or_else(|| {
                if c.sort_value.is_some() {
                    Some(SortUndefined::Dir(1))
                } else {
                    None
                }
            });

            let sort_is_undefined = c
                .sort_is_undefined
                .clone()
                .map(ResolvedIsUndefined::Fn)
                .or_else(|| {
                    if sort_undefined.is_some() {
                        c.sort_value.clone().map(ResolvedIsUndefined::Value)
                    } else {
                        None
                    }
                });

            Some((
                c.id.as_ref(),
                ResolvedSortColumn {
                    cmp,
                    invert_sorting: c.invert_sorting,
                    sort_undefined,
                    sort_is_undefined,
                },
            ))
        })
        .collect();

    let mut out = row_model.clone();

    fn tiebreaker(a: RowKey, b: RowKey) -> Ordering {
        a.cmp(&b)
    }

    fn cmp_rows<TData>(
        arena: &[super::Row<'_, TData>],
        sort_cfg_by_id: &HashMap<&str, ResolvedSortColumn<TData>>,
        sorting: &[SortSpec],
        a: RowIndex,
        b: RowIndex,
    ) -> Ordering {
        let Some(a_row) = arena.get(a) else {
            return Ordering::Equal;
        };
        let Some(b_row) = arena.get(b) else {
            return Ordering::Equal;
        };

        for spec in sorting {
            let Some(cfg) = sort_cfg_by_id.get(spec.column.as_ref()) else {
                continue;
            };

            let mut ord = Ordering::Equal;
            if let Some(sort_undefined) = cfg.sort_undefined {
                if !matches!(sort_undefined, SortUndefined::Disabled) {
                    if let Some(is_undefined) = cfg.sort_is_undefined.as_ref() {
                        let a_undefined = match is_undefined {
                            ResolvedIsUndefined::Fn(f) => f(a_row.original),
                            ResolvedIsUndefined::Value(get_value) => {
                                matches!((get_value)(a_row.original), TanStackValue::Undefined)
                            }
                        };
                        let b_undefined = match is_undefined {
                            ResolvedIsUndefined::Fn(f) => f(b_row.original),
                            ResolvedIsUndefined::Value(get_value) => {
                                matches!((get_value)(b_row.original), TanStackValue::Undefined)
                            }
                        };

                        if a_undefined || b_undefined {
                            ord = match sort_undefined {
                                SortUndefined::First => {
                                    if a_undefined == b_undefined {
                                        Ordering::Equal
                                    } else if a_undefined {
                                        Ordering::Less
                                    } else {
                                        Ordering::Greater
                                    }
                                }
                                SortUndefined::Last => {
                                    if a_undefined == b_undefined {
                                        Ordering::Equal
                                    } else if a_undefined {
                                        Ordering::Greater
                                    } else {
                                        Ordering::Less
                                    }
                                }
                                SortUndefined::Dir(dir) => {
                                    if a_undefined == b_undefined {
                                        Ordering::Equal
                                    } else if a_undefined {
                                        if dir < 0 {
                                            Ordering::Less
                                        } else {
                                            Ordering::Greater
                                        }
                                    } else if dir < 0 {
                                        Ordering::Greater
                                    } else {
                                        Ordering::Less
                                    }
                                }
                                SortUndefined::Disabled => Ordering::Equal,
                            };

                            // TanStack: `first`/`last` returns early (ignores desc/invert multipliers).
                            if matches!(sort_undefined, SortUndefined::First | SortUndefined::Last)
                                && ord != Ordering::Equal
                            {
                                return ord;
                            }
                        }
                    }
                }
            }

            if ord == Ordering::Equal {
                ord = match &cfg.cmp {
                    ResolvedSortCmp::Cmp(cmp) => cmp(a_row.original, b_row.original),
                    ResolvedSortCmp::BuiltIn { kind, get_value } => {
                        compare_builtin_sort(*kind, get_value, a_row.original, b_row.original)
                    }
                };
            }

            if ord != Ordering::Equal {
                if spec.desc {
                    ord = ord.reverse();
                }
                if cfg.invert_sorting {
                    ord = ord.reverse();
                }
                return ord;
            }
        }

        tiebreaker(a_row.key, b_row.key)
    }

    fn sort_children<TData>(
        row_model: &mut RowModel<'_, TData>,
        sort_cfg_by_id: &HashMap<&str, ResolvedSortColumn<TData>>,
        sorting: &[SortSpec],
        row: RowIndex,
    ) {
        let Some(children) = row_model.row(row).map(|r| r.sub_rows.clone()) else {
            return;
        };
        let arena = row_model.arena.as_slice();
        let mut sorted = children;
        sorted.sort_by(|&a, &b| cmp_rows(arena, sort_cfg_by_id, sorting, a, b));

        if let Some(r) = row_model.arena.get_mut(row) {
            r.sub_rows = sorted.clone();
        }

        for child in sorted {
            sort_children(row_model, sort_cfg_by_id, sorting, child);
        }
    }

    let arena = out.arena.as_slice();
    out.root_rows
        .sort_by(|&a, &b| cmp_rows(arena, &sort_cfg_by_id, sorting, a, b));
    let roots = out.root_rows.clone();
    for root in roots {
        sort_children(&mut out, &sort_cfg_by_id, sorting, root);
    }

    out.flat_rows.clear();
    fn push_flat<TData>(row_model: &mut RowModel<'_, TData>, row: RowIndex) {
        row_model.flat_rows.push(row);
        let Some(r) = row_model.row(row) else {
            return;
        };
        let children = r.sub_rows.clone();
        for child in children {
            push_flat(row_model, child);
        }
    }
    let roots = out.root_rows.clone();
    for root in roots {
        push_flat(&mut out, root);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::{Table, TableOptions, create_column_helper};

    #[derive(Debug, Clone)]
    struct Item {
        value: i32,
    }

    #[test]
    fn sort_row_model_sorts_root_rows_by_spec() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let sorting_fns = HashMap::new();
        let sorted = sort_row_model(core, &columns, &sorting, &sorting_fns);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![1, 0, 2]);
    }

    #[test]
    fn sort_row_model_respects_descending() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: true,
        }];

        let sorting_fns = HashMap::new();
        let sorted = sort_row_model(core, &columns, &sorting, &sorting_fns);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![2, 0, 1]);
    }

    #[test]
    fn sort_row_model_uses_row_key_tiebreaker_for_determinism() {
        let data = vec![Item { value: 1 }, Item { value: 1 }, Item { value: 1 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let sorting_fns = HashMap::new();
        let sorted = sort_row_model(core, &columns, &sorting, &sorting_fns);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![0, 1, 2]);
    }

    #[test]
    fn sort_row_model_inverts_sorting_when_column_requests_it() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value).invert_sorting(true)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let sorting_fns = HashMap::new();
        let sorted = sort_row_model(core, &columns, &sorting, &sorting_fns);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![2, 0, 1]);
    }

    #[test]
    fn sort_row_model_invert_sorting_composes_with_descending() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value).invert_sorting(true)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: true,
        }];

        let sorting_fns = HashMap::new();
        let sorted = sort_row_model(core, &columns, &sorting, &sorting_fns);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![1, 0, 2]);
    }

    #[test]
    fn toggle_sort_for_column_cycles_and_resets_when_single() {
        let mut sorting: SortingState = Vec::new();

        toggle_sort_for_column(&mut sorting, "a".into(), false);
        assert_eq!(sorting.len(), 1);
        assert_eq!(sorting[0].column.as_ref(), "a");
        assert!(!sorting[0].desc);

        toggle_sort_for_column(&mut sorting, "a".into(), false);
        assert_eq!(sorting.len(), 1);
        assert!(sorting[0].desc);

        toggle_sort_for_column(&mut sorting, "a".into(), false);
        assert!(sorting.is_empty());

        toggle_sort_for_column(&mut sorting, "b".into(), false);
        assert_eq!(sorting.len(), 1);
        assert_eq!(sorting[0].column.as_ref(), "b");
    }

    #[test]
    fn toggle_sorting_tanstack_respects_sort_desc_first() {
        let mut sorting: SortingState = Vec::new();

        let helper = create_column_helper::<Item>();
        let column = helper
            .accessor("value", |it| it.value)
            .sort_desc_first(true);

        toggle_sorting_tanstack(&mut sorting, &column, TableOptions::default(), false, false);
        assert_eq!(sorting.len(), 1);
        assert_eq!(sorting[0].column.as_ref(), "value");
        assert!(sorting[0].desc);

        toggle_sorting_tanstack(&mut sorting, &column, TableOptions::default(), false, false);
        assert_eq!(sorting.len(), 1);
        assert!(!sorting[0].desc);

        toggle_sorting_tanstack(&mut sorting, &column, TableOptions::default(), false, false);
        assert!(sorting.is_empty());
    }
}
