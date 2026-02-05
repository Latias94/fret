use std::cmp::Ordering;
use std::sync::Arc;

use serde_json::Value;

use super::{Aggregation, AggregationFnSpec, BuiltInAggregationFn};

pub type ColumnId = Arc<str>;

pub type SortCmpFn<TData> = Arc<dyn Fn(&TData, &TData) -> Ordering>;
pub type SortIsUndefinedFn<TData> = Arc<dyn Fn(&TData) -> bool>;
pub type SortValueFn<TData> = Arc<dyn Fn(&TData) -> TanStackValue>;
pub type FilterFn<TData> = Arc<dyn Fn(&TData, &Value) -> bool>;
pub type FacetKeyFn<TData> = Arc<dyn Fn(&TData) -> u64>;
pub type FacetStrFn<TData> = Arc<dyn for<'r> Fn(&'r TData) -> &'r str>;
pub type ValueU64Fn<TData> = Arc<dyn Fn(&TData) -> u64>;

/// A TanStack-like “cell value” representation used by built-in sorting functions.
///
/// This exists because TanStack Table’s built-in sorting functions operate over untyped JS
/// values (including `undefined`). In Rust, we need a stable representation to express those
/// behaviors.
#[derive(Debug, Clone, PartialEq)]
pub enum TanStackValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(Arc<str>),
    Array(Vec<TanStackValue>),
    /// Stored as milliseconds since epoch (JS `Date.valueOf()`).
    DateTime(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInSortingFn {
    Alphanumeric,
    AlphanumericCaseSensitive,
    Text,
    TextCaseSensitive,
    Datetime,
    Basic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortingFnSpec {
    /// TanStack `sortingFn: 'auto'`.
    Auto,
    /// TanStack built-in sorting fn key.
    BuiltIn(BuiltInSortingFn),
    /// TanStack `sortingFn: <string>` resolved via `options.sortingFns[key] ?? builtIn[key]`.
    Named(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInFilterFn {
    IncludesString,
    IncludesStringSensitive,
    EqualsString,
    ArrIncludes,
    ArrIncludesAll,
    ArrIncludesSome,
    Equals,
    WeakEquals,
    InNumberRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilteringFnSpec {
    /// TanStack `filterFn: 'auto'`.
    Auto,
    /// TanStack built-in filter fn key.
    BuiltIn(BuiltInFilterFn),
    /// TanStack `filterFn: <string>` resolved via `options.filterFns[key] ?? builtIn[key]`.
    Named(Arc<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortUndefined {
    /// TanStack `sortUndefined: false` (disable the pre-pass undefined ordering).
    Disabled,
    First,
    Last,
    /// `-1` or `1` in TanStack Table. Applied before `desc`/`invertSorting` multipliers.
    Dir(i8),
}

pub struct ColumnDef<TData> {
    pub id: ColumnId,
    /// Child columns for TanStack-style grouped column definitions.
    ///
    /// When non-empty, this column is treated as a “group” column for header group generation.
    pub columns: Vec<ColumnDef<TData>>,
    pub sort_cmp: Option<SortCmpFn<TData>>,
    pub sorting_fn: Option<SortingFnSpec>,
    pub sort_value: Option<SortValueFn<TData>>,
    pub sort_undefined: Option<SortUndefined>,
    pub sort_is_undefined: Option<SortIsUndefinedFn<TData>>,
    pub filtering_fn: Option<FilteringFnSpec>,
    pub filter_fn: Option<FilterFn<TData>>,
    pub facet_key_fn: Option<FacetKeyFn<TData>>,
    pub facet_str_fn: Option<FacetStrFn<TData>>,
    pub value_u64_fn: Option<ValueU64Fn<TData>>,
    pub invert_sorting: bool,
    pub sort_desc_first: Option<bool>,
    pub enable_sorting: bool,
    pub enable_multi_sort: bool,
    pub enable_column_filter: bool,
    pub enable_global_filter: bool,
    pub aggregation: Aggregation,
    pub aggregation_fn: AggregationFnSpec,
    pub enable_hiding: bool,
    pub enable_ordering: bool,
    pub enable_pinning: bool,
    pub enable_resizing: bool,
    pub enable_grouping: bool,
    pub size: f32,
    pub min_size: f32,
    pub max_size: f32,
}

impl<TData> Clone for ColumnDef<TData> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            columns: self.columns.clone(),
            sort_cmp: self.sort_cmp.clone(),
            sorting_fn: self.sorting_fn.clone(),
            sort_value: self.sort_value.clone(),
            sort_undefined: self.sort_undefined,
            sort_is_undefined: self.sort_is_undefined.clone(),
            filtering_fn: self.filtering_fn.clone(),
            filter_fn: self.filter_fn.clone(),
            facet_key_fn: self.facet_key_fn.clone(),
            facet_str_fn: self.facet_str_fn.clone(),
            value_u64_fn: self.value_u64_fn.clone(),
            invert_sorting: self.invert_sorting,
            sort_desc_first: self.sort_desc_first,
            enable_sorting: self.enable_sorting,
            enable_multi_sort: self.enable_multi_sort,
            enable_column_filter: self.enable_column_filter,
            enable_global_filter: self.enable_global_filter,
            aggregation: self.aggregation,
            aggregation_fn: self.aggregation_fn.clone(),
            enable_hiding: self.enable_hiding,
            enable_ordering: self.enable_ordering,
            enable_pinning: self.enable_pinning,
            enable_resizing: self.enable_resizing,
            enable_grouping: self.enable_grouping,
            size: self.size,
            min_size: self.min_size,
            max_size: self.max_size,
        }
    }
}

impl<TData> std::fmt::Debug for ColumnDef<TData> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ColumnDef")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<TData> ColumnDef<TData> {
    pub fn new(id: impl Into<ColumnId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            sort_cmp: None,
            sorting_fn: None,
            sort_value: None,
            sort_undefined: None,
            sort_is_undefined: None,
            filtering_fn: None,
            filter_fn: None,
            facet_key_fn: None,
            facet_str_fn: None,
            value_u64_fn: None,
            invert_sorting: false,
            sort_desc_first: None,
            enable_sorting: true,
            enable_multi_sort: true,
            enable_column_filter: true,
            enable_global_filter: true,
            aggregation: Aggregation::None,
            aggregation_fn: AggregationFnSpec::Auto,
            enable_hiding: true,
            enable_ordering: true,
            enable_pinning: true,
            enable_resizing: true,
            enable_grouping: true,
            size: 150.0,
            min_size: 20.0,
            max_size: f32::MAX,
        }
    }

    pub fn sort_by(mut self, cmp: impl Fn(&TData, &TData) -> Ordering + 'static) -> Self {
        self.sort_cmp = Some(Arc::new(cmp));
        self
    }

    /// TanStack-aligned: configure `columns` for grouped column definitions.
    pub fn columns(mut self, columns: Vec<ColumnDef<TData>>) -> Self {
        self.columns = columns;
        self
    }

    /// Provide a TanStack-like `getValue(columnId)` accessor for built-in sortingFn behaviors.
    pub fn sort_value_by(mut self, get_value: impl Fn(&TData) -> TanStackValue + 'static) -> Self {
        self.sort_value = Some(Arc::new(get_value));
        self
    }

    /// TanStack-aligned: configure `aggregationFn: 'auto'`.
    pub fn aggregation_fn_auto(mut self) -> Self {
        self.aggregation_fn = AggregationFnSpec::Auto;
        self
    }

    /// TanStack-aligned: configure a built-in aggregation function key.
    pub fn aggregation_fn_builtin(mut self, agg: BuiltInAggregationFn) -> Self {
        self.aggregation_fn = AggregationFnSpec::BuiltIn(agg);
        self
    }

    /// TanStack-aligned: configure `aggregationFn: <string>` resolved via `options.aggregationFns`.
    pub fn aggregation_fn_named(mut self, key: impl Into<Arc<str>>) -> Self {
        self.aggregation_fn = AggregationFnSpec::Named(key.into());
        self
    }

    /// Disable aggregation for this column.
    pub fn aggregation_fn_none(mut self) -> Self {
        self.aggregation_fn = AggregationFnSpec::None;
        self
    }

    /// TanStack-aligned: configure `sortingFn: 'auto'`.
    pub fn sorting_fn_auto(mut self) -> Self {
        self.sorting_fn = Some(SortingFnSpec::Auto);
        self
    }

    /// TanStack-aligned: configure a built-in sorting function key.
    pub fn sorting_fn_builtin(mut self, sorting_fn: BuiltInSortingFn) -> Self {
        self.sorting_fn = Some(SortingFnSpec::BuiltIn(sorting_fn));
        self
    }

    /// TanStack-aligned: configure `sortingFn: <string>` resolved via table options.
    pub fn sorting_fn_named(mut self, key: impl Into<Arc<str>>) -> Self {
        self.sorting_fn = Some(SortingFnSpec::Named(key.into()));
        self
    }

    /// TanStack-aligned: configure `filterFn: 'auto'`.
    pub fn filtering_fn_auto(mut self) -> Self {
        self.filtering_fn = Some(FilteringFnSpec::Auto);
        self
    }

    /// TanStack-aligned: configure a built-in filter function key.
    pub fn filtering_fn_builtin(mut self, filter_fn: BuiltInFilterFn) -> Self {
        self.filtering_fn = Some(FilteringFnSpec::BuiltIn(filter_fn));
        self
    }

    /// TanStack-aligned: configure `filterFn: <string>` resolved via table options.
    pub fn filtering_fn_named(mut self, key: impl Into<Arc<str>>) -> Self {
        self.filtering_fn = Some(FilteringFnSpec::Named(key.into()));
        self
    }

    /// TanStack-aligned: configure `sortUndefined` semantics for this column.
    ///
    /// `is_undefined` must match the column's `getValue(column_id) === undefined` behavior in
    /// TanStack.
    pub fn sort_undefined_by(
        mut self,
        sort_undefined: SortUndefined,
        is_undefined: impl Fn(&TData) -> bool + 'static,
    ) -> Self {
        self.sort_undefined = Some(sort_undefined);
        self.sort_is_undefined = Some(Arc::new(is_undefined));
        self
    }

    /// TanStack-aligned: `sortUndefined: false` (disable undefined pre-pass ordering).
    pub fn sort_undefined_disabled(mut self) -> Self {
        self.sort_undefined = Some(SortUndefined::Disabled);
        self.sort_is_undefined = None;
        self
    }

    /// TanStack-aligned: invert the meaning of `asc` vs `desc` for this column.
    ///
    /// This mirrors `columnDef.invertSorting` in TanStack Table v8: after the base sorting
    /// function yields an ordering, the result is inverted.
    pub fn invert_sorting(mut self, invert: bool) -> Self {
        self.invert_sorting = invert;
        self
    }

    /// TanStack-aligned: start sort toggles in descending order for this column.
    ///
    /// This mirrors `columnDef.sortDescFirst` in TanStack Table v8.
    pub fn sort_desc_first(mut self, enabled: bool) -> Self {
        self.sort_desc_first = Some(enabled);
        self
    }

    /// TanStack-aligned: enable/disable sorting for this column.
    ///
    /// This mirrors `columnDef.enableSorting` in TanStack Table v8.
    pub fn enable_sorting(mut self, enabled: bool) -> Self {
        self.enable_sorting = enabled;
        self
    }

    /// TanStack-aligned: enable/disable multi-sort for this column.
    ///
    /// This mirrors `columnDef.enableMultiSort` in TanStack Table v8.
    pub fn enable_multi_sort(mut self, enabled: bool) -> Self {
        self.enable_multi_sort = enabled;
        self
    }

    pub fn enable_column_filter(mut self, enabled: bool) -> Self {
        self.enable_column_filter = enabled;
        self
    }

    pub fn enable_global_filter(mut self, enabled: bool) -> Self {
        self.enable_global_filter = enabled;
        self
    }

    pub fn filter_by(mut self, f: impl Fn(&TData, &str) -> bool + 'static) -> Self {
        let f = Arc::new(f);
        self.filter_fn = Some(Arc::new(move |row, value| {
            let Some(s) = value.as_str() else {
                return false;
            };
            f(row, s)
        }));
        self
    }

    /// Provide a stable `u64` facet key for this column (TanStack-aligned faceting, Rust-native).
    pub fn facet_key_by(mut self, f: impl Fn(&TData) -> u64 + 'static) -> Self {
        self.facet_key_fn = Some(Arc::new(f));
        self
    }

    /// Provide a string view for this column's facet value (borrowed from row data; no allocation).
    pub fn facet_str_by(mut self, f: impl for<'r> Fn(&'r TData) -> &'r str + 'static) -> Self {
        self.facet_str_fn = Some(Arc::new(f));
        self
    }

    /// Provide a stable numeric value for this column.
    ///
    /// This is the preferred input for numeric aggregation (and future numeric sorting/filtering).
    /// It is intentionally separate from `facet_key_by`, which is reserved for grouping/faceting.
    pub fn value_u64_by(mut self, f: impl Fn(&TData) -> u64 + 'static) -> Self {
        self.value_u64_fn = Some(Arc::new(f));
        self
    }

    pub fn aggregate(mut self, aggregation: Aggregation) -> Self {
        self.aggregation = aggregation;
        self
    }

    pub fn enable_hiding(mut self, enabled: bool) -> Self {
        self.enable_hiding = enabled;
        self
    }

    pub fn enable_ordering(mut self, enabled: bool) -> Self {
        self.enable_ordering = enabled;
        self
    }

    pub fn enable_pinning(mut self, enabled: bool) -> Self {
        self.enable_pinning = enabled;
        self
    }

    pub fn enable_resizing(mut self, enabled: bool) -> Self {
        self.enable_resizing = enabled;
        self
    }

    pub fn enable_grouping(mut self, enabled: bool) -> Self {
        self.enable_grouping = enabled;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn min_size(mut self, min_size: f32) -> Self {
        self.min_size = min_size;
        self
    }

    pub fn max_size(mut self, max_size: f32) -> Self {
        self.max_size = max_size;
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColumnHelper<TData> {
    _marker: std::marker::PhantomData<TData>,
}

pub fn create_column_helper<TData>() -> ColumnHelper<TData> {
    ColumnHelper {
        _marker: std::marker::PhantomData,
    }
}

impl<TData> ColumnHelper<TData> {
    pub fn accessor<V>(
        self,
        id: impl Into<ColumnId>,
        accessor: impl Fn(&TData) -> V + 'static,
    ) -> ColumnDef<TData>
    where
        V: Ord,
    {
        let accessor = Arc::new(accessor);
        ColumnDef::new(id).sort_by(move |a, b| accessor(a).cmp(&accessor(b)))
    }

    pub fn accessor_str(
        self,
        id: impl Into<ColumnId>,
        accessor: impl for<'r> Fn(&'r TData) -> &'r str + 'static,
    ) -> ColumnDef<TData>
    where
        TData: 'static,
    {
        let accessor: Arc<dyn for<'r> Fn(&'r TData) -> &'r str> = Arc::new(accessor);
        let sort_accessor = accessor.clone();
        let facet_accessor = accessor.clone();
        ColumnDef::new(id)
            .sort_by(move |a, b| sort_accessor(a).cmp(sort_accessor(b)))
            .facet_str_by(move |row| facet_accessor(row))
    }
}
