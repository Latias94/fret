use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

/// Stable identity for a row in the table.
///
/// This is aligned with TanStack Table's `getRowId` guidance, but uses an efficient numeric key so
/// it can be used in hot paths (selection, row maps, virtualization keys) without heap allocation.
///
/// The default key strategy is index-path based, so callers should supply their own stable key
/// (e.g. a database primary key) when the underlying data can reorder or change over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RowKey(pub u64);

impl RowKey {
    pub fn from_index(index: usize) -> Self {
        Self(index as u64)
    }
}

/// Index into a [`RowModel`] arena.
pub type RowIndex = usize;

#[derive(Debug)]
pub struct Row<'a, TData> {
    pub key: RowKey,
    pub original: &'a TData,
    pub index: usize,
    pub depth: u16,
    pub parent: Option<RowIndex>,
    pub parent_key: Option<RowKey>,
    pub sub_rows: Vec<RowIndex>,
}

impl<'a, TData> Clone for Row<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            original: self.original,
            index: self.index,
            depth: self.depth,
            parent: self.parent,
            parent_key: self.parent_key,
            sub_rows: self.sub_rows.clone(),
        }
    }
}

#[derive(Debug)]
pub struct RowModel<'a, TData> {
    pub(super) root_rows: Vec<RowIndex>,
    pub(super) flat_rows: Vec<RowIndex>,
    pub(super) rows_by_key: HashMap<RowKey, RowIndex>,
    pub(super) arena: Vec<Row<'a, TData>>,
}

impl<'a, TData> Clone for RowModel<'a, TData> {
    fn clone(&self) -> Self {
        Self {
            root_rows: self.root_rows.clone(),
            flat_rows: self.flat_rows.clone(),
            rows_by_key: self.rows_by_key.clone(),
            arena: self.arena.clone(),
        }
    }
}

impl<'a, TData> RowModel<'a, TData> {
    pub fn root_rows(&self) -> &[RowIndex] {
        &self.root_rows
    }

    pub fn flat_rows(&self) -> &[RowIndex] {
        &self.flat_rows
    }

    pub fn row(&self, index: RowIndex) -> Option<&Row<'a, TData>> {
        self.arena.get(index)
    }

    pub fn row_by_key(&self, key: RowKey) -> Option<RowIndex> {
        self.rows_by_key.get(&key).copied()
    }

    pub fn rows_by_key(&self) -> &HashMap<RowKey, RowIndex> {
        &self.rows_by_key
    }

    pub fn arena(&self) -> &[Row<'a, TData>] {
        &self.arena
    }
}

type GetRowKeyFn<'a, TData> = Box<dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'a>;
type GetSubRowsFn<'a, TData> = Box<dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]> + 'a>;

pub struct TableBuilder<'a, TData> {
    data: &'a [TData],
    columns: Vec<super::ColumnDef<TData>>,
    sorting_fns: HashMap<Arc<str>, super::SortingFnDef<TData>>,
    filter_fns: HashMap<Arc<str>, super::FilterFnDef>,
    global_filter_fn: super::FilteringFnSpec,
    get_column_can_global_filter: Option<Arc<dyn Fn(&super::ColumnDef<TData>, &TData) -> bool>>,
    get_row_key: Option<GetRowKeyFn<'a, TData>>,
    get_sub_rows: Option<GetSubRowsFn<'a, TData>>,
    state: super::TableState,
    options: super::TableOptions,
}

impl<'a, TData> TableBuilder<'a, TData> {
    pub fn new(data: &'a [TData]) -> Self {
        Self {
            data,
            columns: Vec::new(),
            sorting_fns: HashMap::new(),
            filter_fns: HashMap::new(),
            global_filter_fn: super::FilteringFnSpec::Auto,
            get_column_can_global_filter: None,
            get_row_key: None,
            get_sub_rows: None,
            state: super::TableState::default(),
            options: super::TableOptions::default(),
        }
    }

    pub fn columns(mut self, columns: Vec<super::ColumnDef<TData>>) -> Self {
        self.columns = columns;
        self
    }

    /// Register a named sorting function (TanStack `options.sortingFns` equivalent).
    pub fn sorting_fn_builtin(
        mut self,
        key: impl Into<Arc<str>>,
        sorting_fn: super::BuiltInSortingFn,
    ) -> Self {
        self.sorting_fns
            .insert(key.into(), super::SortingFnDef::BuiltIn(sorting_fn));
        self
    }

    /// Register a named sorting comparator (TanStack `options.sortingFns` equivalent).
    pub fn sorting_fn_cmp(
        mut self,
        key: impl Into<Arc<str>>,
        cmp: impl Fn(&TData, &TData) -> std::cmp::Ordering + 'static,
    ) -> Self {
        self.sorting_fns
            .insert(key.into(), super::SortingFnDef::Cmp(Arc::new(cmp)));
        self
    }

    /// Register a named filter function (TanStack `options.filterFns` equivalent).
    pub fn filter_fn_builtin(
        mut self,
        key: impl Into<Arc<str>>,
        filter_fn: super::BuiltInFilterFn,
    ) -> Self {
        self.filter_fns
            .insert(key.into(), super::FilterFnDef::BuiltIn(filter_fn));
        self
    }

    /// Register a named filter function that operates over the column's `getValue()`.
    pub fn filter_fn_value(
        mut self,
        key: impl Into<Arc<str>>,
        f: impl Fn(&super::TanStackValue, &serde_json::Value) -> bool + 'static,
    ) -> Self {
        self.filter_fns
            .insert(key.into(), super::FilterFnDef::Value(Arc::new(f)));
        self
    }

    /// Configure the global filter function (TanStack `globalFilterFn`).
    pub fn global_filter_fn(mut self, spec: super::FilteringFnSpec) -> Self {
        self.global_filter_fn = spec;
        self
    }

    /// Configure the table-level “can global filter” hook (TanStack `getColumnCanGlobalFilter`).
    pub fn get_column_can_global_filter(
        mut self,
        f: impl Fn(&super::ColumnDef<TData>, &TData) -> bool + 'static,
    ) -> Self {
        self.get_column_can_global_filter = Some(Arc::new(f));
        self
    }

    pub fn state(mut self, state: super::TableState) -> Self {
        self.state = state;
        self
    }

    pub fn options(mut self, options: super::TableOptions) -> Self {
        self.options = options;
        self
    }

    pub fn manual_filtering(mut self, manual: bool) -> Self {
        self.options.manual_filtering = manual;
        self
    }

    pub fn manual_sorting(mut self, manual: bool) -> Self {
        self.options.manual_sorting = manual;
        self
    }

    pub fn manual_pagination(mut self, manual: bool) -> Self {
        self.options.manual_pagination = manual;
        self
    }

    pub fn manual_expanding(mut self, manual: bool) -> Self {
        self.options.manual_expanding = manual;
        self
    }

    pub fn paginate_expanded_rows(mut self, enabled: bool) -> Self {
        self.options.paginate_expanded_rows = enabled;
        self
    }

    pub fn keep_pinned_rows(mut self, keep: bool) -> Self {
        self.options.keep_pinned_rows = keep;
        self
    }

    pub fn enable_hiding(mut self, enabled: bool) -> Self {
        self.options.enable_hiding = enabled;
        self
    }

    pub fn enable_column_ordering(mut self, enabled: bool) -> Self {
        self.options.enable_column_ordering = enabled;
        self
    }

    pub fn enable_column_pinning(mut self, enabled: bool) -> Self {
        self.options.enable_column_pinning = enabled;
        self
    }

    pub fn enable_column_resizing(mut self, enabled: bool) -> Self {
        self.options.enable_column_resizing = enabled;
        self
    }

    pub fn get_row_key(
        mut self,
        f: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'a,
    ) -> Self {
        self.get_row_key = Some(Box::new(f));
        self
    }

    pub fn get_sub_rows(
        mut self,
        f: impl for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]> + 'a,
    ) -> Self {
        self.get_sub_rows = Some(Box::new(f));
        self
    }

    pub fn build(self) -> Table<'a, TData> {
        Table::new(self)
    }
}

pub struct Table<'a, TData> {
    data: &'a [TData],
    column_tree: Vec<super::ColumnDef<TData>>,
    columns: Vec<super::ColumnDef<TData>>,
    sorting_fns: HashMap<Arc<str>, super::SortingFnDef<TData>>,
    filter_fns: HashMap<Arc<str>, super::FilterFnDef>,
    global_filter_fn: super::FilteringFnSpec,
    get_column_can_global_filter: Option<Arc<dyn Fn(&super::ColumnDef<TData>, &TData) -> bool>>,
    get_row_key: GetRowKeyFn<'a, TData>,
    get_sub_rows: Option<GetSubRowsFn<'a, TData>>,
    state: super::TableState,
    options: super::TableOptions,
    core_row_model: OnceCell<RowModel<'a, TData>>,
    filtered_row_model: OnceCell<RowModel<'a, TData>>,
    grouped_row_model: OnceCell<super::GroupedRowModel>,
    sorted_row_model: OnceCell<RowModel<'a, TData>>,
    expanded_row_model: OnceCell<RowModel<'a, TData>>,
    paginated_row_model: OnceCell<RowModel<'a, TData>>,
    expanded_paginated_row_model: OnceCell<RowModel<'a, TData>>,
    selected_row_model: OnceCell<RowModel<'a, TData>>,
    filtered_selected_row_model: OnceCell<RowModel<'a, TData>>,
    grouped_selected_row_model: OnceCell<RowModel<'a, TData>>,
    page_selected_row_model: OnceCell<RowModel<'a, TData>>,
    faceted_row_model_by_column: OnceCell<Vec<OnceCell<RowModel<'a, TData>>>>,
    faceted_unique_values_by_column: OnceCell<Vec<OnceCell<super::FacetCounts>>>,
    faceted_unique_labels_by_column: OnceCell<Vec<OnceCell<super::FacetLabels<'a>>>>,
    faceted_min_max_u64_by_column: OnceCell<Vec<OnceCell<Option<(u64, u64)>>>>,
}

fn rebuild_flat_rows_from_roots_including_duplicates<TData>(row_model: &mut RowModel<'_, TData>) {
    // TanStack Table v8 `getPaginationRowModel` rebuilds `flatRows` by recursively visiting each row
    // from the paginated `rows` list and then traversing `subRows` unconditionally. When
    // `paginateExpandedRows=false`, those paginated `rows` can already include expanded descendants,
    // which results in duplicated `flatRows` entries.
    let roots = row_model.root_rows.clone();
    let mut flat = Vec::new();

    fn push_flat<TData>(arena: &[Row<'_, TData>], out: &mut Vec<RowIndex>, row: RowIndex) {
        out.push(row);
        let Some(r) = arena.get(row) else {
            return;
        };
        for &child in &r.sub_rows {
            push_flat(arena, out, child);
        }
    }

    for root in roots {
        push_flat(&row_model.arena, &mut flat, root);
    }

    row_model.flat_rows = flat;
}

impl<'a, TData> Table<'a, TData> {
    pub fn builder(data: &'a [TData]) -> TableBuilder<'a, TData> {
        TableBuilder::new(data)
    }

    fn new(builder: TableBuilder<'a, TData>) -> Self {
        fn push_leaf_columns<TData>(
            cols: &[super::ColumnDef<TData>],
            out: &mut Vec<super::ColumnDef<TData>>,
        ) {
            for col in cols {
                if col.columns.is_empty() {
                    out.push(col.clone());
                } else {
                    push_leaf_columns(&col.columns, out);
                }
            }
        }

        let get_row_key = builder
            .get_row_key
            .unwrap_or_else(|| Box::new(default_row_key_for_index_path));

        let column_tree = builder.columns;
        let mut columns: Vec<super::ColumnDef<TData>> = Vec::new();
        push_leaf_columns(&column_tree, &mut columns);

        Self {
            data: builder.data,
            column_tree,
            columns,
            sorting_fns: builder.sorting_fns,
            filter_fns: builder.filter_fns,
            global_filter_fn: builder.global_filter_fn,
            get_column_can_global_filter: builder.get_column_can_global_filter,
            get_row_key,
            get_sub_rows: builder.get_sub_rows,
            state: builder.state,
            options: builder.options,
            core_row_model: OnceCell::new(),
            filtered_row_model: OnceCell::new(),
            grouped_row_model: OnceCell::new(),
            sorted_row_model: OnceCell::new(),
            expanded_row_model: OnceCell::new(),
            paginated_row_model: OnceCell::new(),
            expanded_paginated_row_model: OnceCell::new(),
            selected_row_model: OnceCell::new(),
            filtered_selected_row_model: OnceCell::new(),
            grouped_selected_row_model: OnceCell::new(),
            page_selected_row_model: OnceCell::new(),
            faceted_row_model_by_column: OnceCell::new(),
            faceted_unique_values_by_column: OnceCell::new(),
            faceted_unique_labels_by_column: OnceCell::new(),
            faceted_min_max_u64_by_column: OnceCell::new(),
        }
    }

    pub fn data(&self) -> &'a [TData] {
        self.data
    }

    pub fn columns(&self) -> &[super::ColumnDef<TData>] {
        &self.columns
    }

    pub fn column_tree(&self) -> &[super::ColumnDef<TData>] {
        &self.column_tree
    }

    pub fn column(&self, id: &str) -> Option<&super::ColumnDef<TData>> {
        self.columns.iter().find(|c| c.id.as_ref() == id)
    }

    fn column_index(&self, id: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.id.as_ref() == id)
    }

    /// TanStack-aligned: resolve a row by id, optionally searching outside the current paginated
    /// row model (e.g. pinned rows).
    pub fn row(&self, row_key: RowKey, search_all: bool) -> Option<&Row<'a, TData>> {
        let first = if search_all {
            self.pre_pagination_row_model()
        } else {
            self.row_model()
        };

        first
            .row_by_key(row_key)
            .and_then(|i| first.row(i))
            .or_else(|| {
                let core = self.core_row_model();
                core.row_by_key(row_key).and_then(|i| core.row(i))
            })
    }

    pub fn state(&self) -> &super::TableState {
        &self.state
    }

    pub fn options(&self) -> super::TableOptions {
        self.options
    }

    pub fn column_visibility(&self) -> &super::ColumnVisibilityState {
        &self.state.column_visibility
    }

    pub fn is_column_visible(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(super::is_column_visible(
            &self.state.column_visibility,
            &col.id,
        ))
    }

    pub fn column_can_hide(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(self.options.enable_hiding && col.enable_hiding)
    }

    pub fn hideable_columns(&self) -> Vec<&super::ColumnDef<TData>> {
        self.ordered_columns()
            .into_iter()
            .filter(|c| self.options.enable_hiding && c.enable_hiding)
            .collect()
    }

    pub fn is_all_columns_visible(&self) -> bool {
        self.columns
            .iter()
            .all(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
    }

    pub fn is_some_columns_visible(&self) -> bool {
        self.columns
            .iter()
            .any(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
    }

    pub fn toggled_column_visibility(
        &self,
        column_id: &str,
        visible: Option<bool>,
    ) -> Option<super::ColumnVisibilityState> {
        let col = self.column(column_id)?;
        if !(self.options.enable_hiding && col.enable_hiding) {
            return Some(self.state.column_visibility.clone());
        }
        Some(super::toggled_column_visible(
            &self.state.column_visibility,
            &col.id,
            visible,
        ))
    }

    pub fn toggled_all_columns_visible(
        &self,
        visible: Option<bool>,
    ) -> super::ColumnVisibilityState {
        let visible = visible.unwrap_or_else(|| !self.is_all_columns_visible());

        let mut next = self.state.column_visibility.clone();
        for col in &self.columns {
            let can_hide = self.options.enable_hiding && col.enable_hiding;
            if visible {
                super::set_column_visible(&mut next, &col.id, true);
            } else {
                super::set_column_visible(&mut next, &col.id, !can_hide);
            }
        }
        next
    }

    pub fn is_some_rows_expanded(&self) -> bool {
        super::is_some_rows_expanded(&self.state.expanding)
    }

    pub fn toggled_all_rows_expanded(&self, value: Option<bool>) -> super::ExpandingState {
        let value = value.unwrap_or_else(|| !self.is_all_rows_expanded());
        let mut next = self.state.expanding.clone();
        super::set_all_rows_expanded(&mut next, value);
        next
    }

    pub fn toggled_row_expanded(
        &self,
        row_key: RowKey,
        value: Option<bool>,
    ) -> super::ExpandingState {
        let mut next = self.state.expanding.clone();
        super::toggle_row_expanded(&mut next, self.row_model(), row_key, value);
        next
    }

    pub fn grouping(&self) -> &super::GroupingState {
        &self.state.grouping
    }

    pub fn column_can_group(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(super::column_can_group(self.options, col))
    }

    pub fn is_column_grouped(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(super::is_column_grouped(&self.state.grouping, &col.id))
    }

    pub fn column_grouped_index(&self, column_id: &str) -> Option<usize> {
        let col = self.column(column_id)?;
        super::grouped_index(&self.state.grouping, &col.id)
    }

    pub fn toggled_column_grouping(
        &self,
        column_id: &str,
        grouped: Option<bool>,
    ) -> Option<super::GroupingState> {
        let col = self.column(column_id)?;
        if !super::column_can_group(self.options, col) {
            return Some(self.state.grouping.clone());
        }
        Some(super::toggled_column_grouping_value(
            &self.state.grouping,
            &col.id,
            grouped,
        ))
    }

    pub fn pre_grouped_row_model(&self) -> &RowModel<'a, TData> {
        self.filtered_row_model()
    }

    pub fn grouped_row_model(&self) -> &super::GroupedRowModel {
        if self.options.manual_grouping || self.state.grouping.is_empty() {
            return self
                .grouped_row_model
                .get_or_init(|| super::grouped_row_model_from_leaf(self.pre_grouped_row_model()));
        }

        self.grouped_row_model.get_or_init(|| {
            super::group_row_model(
                self.pre_grouped_row_model(),
                &self.columns,
                &self.state.grouping,
            )
        })
    }

    pub fn is_all_rows_expanded(&self) -> bool {
        match &self.state.expanding {
            super::ExpandingState::All => true,
            super::ExpandingState::Keys(keys) if keys.is_empty() => false,
            _ => {
                let model = self.row_model();
                model
                    .flat_rows()
                    .iter()
                    .filter_map(|&i| model.row(i))
                    .all(|r| super::is_row_expanded(r.key, &self.state.expanding))
            }
        }
    }

    pub fn can_some_rows_expand(&self) -> bool {
        let model = self.pre_pagination_row_model();
        model
            .flat_rows()
            .iter()
            .any(|&i| super::row_can_expand(model, i))
    }

    pub fn expanded_depth(&self) -> u16 {
        super::expanded_depth(self.pre_expanded_row_model(), &self.state.expanding)
    }

    pub fn row_can_expand(&self, row_key: RowKey) -> bool {
        let model = self.pre_expanded_row_model();
        model
            .row_by_key(row_key)
            .is_some_and(|i| super::row_can_expand(model, i))
    }

    pub fn row_is_all_parents_expanded(&self, row_key: RowKey) -> bool {
        let model = self.pre_expanded_row_model();
        model
            .row_by_key(row_key)
            .is_some_and(|i| super::row_is_all_parents_expanded(model, &self.state.expanding, i))
    }

    pub fn is_some_rows_pinned(&self, position: Option<super::RowPinPosition>) -> bool {
        super::is_some_rows_pinned(&self.state.row_pinning, position)
    }

    pub fn is_some_columns_pinned(&self, position: Option<super::ColumnPinPosition>) -> bool {
        super::is_some_columns_pinned(&self.state.column_pinning, position)
    }

    pub fn row_is_pinned(&self, row_key: RowKey) -> Option<super::RowPinPosition> {
        super::is_row_pinned(row_key, &self.state.row_pinning)
    }

    pub fn top_row_keys(&self) -> Vec<RowKey> {
        self.pinned_row_keys(super::RowPinPosition::Top)
    }

    pub fn bottom_row_keys(&self) -> Vec<RowKey> {
        self.pinned_row_keys(super::RowPinPosition::Bottom)
    }

    pub fn center_row_keys(&self) -> Vec<RowKey> {
        let model = self.row_model();
        super::center_row_keys(model.root_rows(), model, &self.state.row_pinning)
    }

    fn pinned_row_keys(&self, position: super::RowPinPosition) -> Vec<RowKey> {
        let keys = match position {
            super::RowPinPosition::Top => self.state.row_pinning.top.as_slice(),
            super::RowPinPosition::Bottom => self.state.row_pinning.bottom.as_slice(),
        };
        if keys.is_empty() {
            return Vec::new();
        }

        if !self.options.keep_pinned_rows {
            let model = self.row_model();
            let visible: std::collections::HashSet<RowKey> = model
                .root_rows()
                .iter()
                .filter_map(|&i| model.row(i).map(|r| r.key))
                .collect();
            return keys
                .iter()
                .copied()
                .filter(|k| visible.contains(k))
                .collect();
        }

        let core = self.core_row_model();
        keys.iter()
            .copied()
            .filter(|k| {
                core.row_by_key(*k).is_some_and(|i| {
                    super::row_is_all_parents_expanded(core, &self.state.expanding, i)
                })
            })
            .collect()
    }

    pub fn ordered_columns(&self) -> Vec<&super::ColumnDef<TData>> {
        super::order_columns(&self.columns, &self.state.column_order)
    }

    pub fn column_order(&self) -> &super::ColumnOrderState {
        &self.state.column_order
    }

    pub fn column_pinning(&self) -> &super::ColumnPinningState {
        &self.state.column_pinning
    }

    pub fn column_can_order(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(self.options.enable_column_ordering && col.enable_ordering)
    }

    pub fn column_can_pin(&self, column_id: &str) -> Option<bool> {
        let col = self.column(column_id)?;
        Some(self.options.enable_column_pinning && col.enable_pinning)
    }

    pub fn column_pin_position(&self, column_id: &str) -> Option<super::ColumnPinPosition> {
        let col = self.column(column_id)?;
        super::is_column_pinned(&self.state.column_pinning, &col.id)
    }

    pub fn toggled_column_order_move(
        &self,
        column_id: &str,
        to_index: usize,
    ) -> Option<super::ColumnOrderState> {
        let col = self.column(column_id)?;
        if !(self.options.enable_column_ordering && col.enable_ordering) {
            return Some(self.state.column_order.clone());
        }
        Some(super::moved_column(
            &self.state.column_order,
            &col.id,
            to_index,
        ))
    }

    pub fn toggled_column_pinning(
        &self,
        column_id: &str,
        position: Option<super::ColumnPinPosition>,
    ) -> Option<super::ColumnPinningState> {
        let col = self.column(column_id)?;
        if !(self.options.enable_column_pinning && col.enable_pinning) {
            return Some(self.state.column_pinning.clone());
        }
        Some(super::pinned_column(
            &self.state.column_pinning,
            &col.id,
            position,
        ))
    }

    pub fn visible_columns(&self) -> Vec<&super::ColumnDef<TData>> {
        self.ordered_columns()
            .into_iter()
            .filter(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
            .collect()
    }

    pub fn header_groups(&self) -> Vec<super::HeaderGroupSnapshot> {
        let (left, center, right) = self.pinned_visible_columns();
        let mut columns_to_group: Vec<std::sync::Arc<str>> = Vec::new();
        columns_to_group.extend(left.into_iter().map(|c| c.id.clone()));
        columns_to_group.extend(center.into_iter().map(|c| c.id.clone()));
        columns_to_group.extend(right.into_iter().map(|c| c.id.clone()));

        let leaf_visible = |id: &str| {
            self.column(id)
                .is_some_and(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
        };

        super::build_header_groups(&self.column_tree, &columns_to_group, &leaf_visible, None)
    }

    pub fn left_header_groups(&self) -> Vec<super::HeaderGroupSnapshot> {
        let (left, _, _) = self.pinned_visible_columns();
        let columns_to_group: Vec<std::sync::Arc<str>> =
            left.into_iter().map(|c| c.id.clone()).collect();
        let leaf_visible = |id: &str| {
            self.column(id)
                .is_some_and(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
        };
        super::build_header_groups(
            &self.column_tree,
            &columns_to_group,
            &leaf_visible,
            Some("left"),
        )
    }

    pub fn center_header_groups(&self) -> Vec<super::HeaderGroupSnapshot> {
        let (_, center, _) = self.pinned_visible_columns();
        let columns_to_group: Vec<std::sync::Arc<str>> =
            center.into_iter().map(|c| c.id.clone()).collect();
        let leaf_visible = |id: &str| {
            self.column(id)
                .is_some_and(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
        };
        super::build_header_groups(
            &self.column_tree,
            &columns_to_group,
            &leaf_visible,
            Some("center"),
        )
    }

    pub fn right_header_groups(&self) -> Vec<super::HeaderGroupSnapshot> {
        let (_, _, right) = self.pinned_visible_columns();
        let columns_to_group: Vec<std::sync::Arc<str>> =
            right.into_iter().map(|c| c.id.clone()).collect();
        let leaf_visible = |id: &str| {
            self.column(id)
                .is_some_and(|c| super::is_column_visible(&self.state.column_visibility, &c.id))
        };
        super::build_header_groups(
            &self.column_tree,
            &columns_to_group,
            &leaf_visible,
            Some("right"),
        )
    }

    pub fn row_cells(&self, row_key: RowKey) -> Option<super::RowCellsSnapshot> {
        let row = self.row(row_key, true)?;
        let row_id = row.key.0.to_string();

        let all_leaf_columns = self.ordered_columns();
        let (left, center, right) = self.pinned_visible_columns();

        Some(super::snapshot_cells_for_row(
            &row_id,
            all_leaf_columns.as_slice(),
            left.as_slice(),
            center.as_slice(),
            right.as_slice(),
        ))
    }

    pub fn core_model_snapshot(&self) -> super::CoreModelSnapshot {
        fn push_column_nodes<TData>(
            cols: &[super::ColumnDef<TData>],
            depth: usize,
            parent_id: Option<Arc<str>>,
            out: &mut Vec<super::ColumnNodeSnapshot>,
        ) {
            for col in cols {
                let id = col.id.clone();
                let child_ids: Vec<Arc<str>> = col.columns.iter().map(|c| c.id.clone()).collect();
                out.push(super::ColumnNodeSnapshot {
                    id: id.clone(),
                    depth,
                    parent_id: parent_id.clone(),
                    child_ids: child_ids.clone(),
                });
                if !col.columns.is_empty() {
                    push_column_nodes(&col.columns, depth + 1, Some(id), out);
                }
            }
        }

        fn snapshot_row_model_ids<'a, TData>(
            model: &RowModel<'a, TData>,
        ) -> super::RowModelIdSnapshot {
            let root: Vec<Arc<str>> = model
                .root_rows()
                .iter()
                .filter_map(|&i| model.row(i).map(|r| Arc::<str>::from(r.key.0.to_string())))
                .collect();
            let flat: Vec<Arc<str>> = model
                .flat_rows()
                .iter()
                .filter_map(|&i| model.row(i).map(|r| Arc::<str>::from(r.key.0.to_string())))
                .collect();
            super::RowModelIdSnapshot { root, flat }
        }

        let mut column_tree = Vec::new();
        push_column_nodes(&self.column_tree, 0, None, &mut column_tree);

        let all_leaf = self
            .ordered_columns()
            .into_iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let visible = self
            .visible_columns()
            .into_iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let (left, center, right) = self.pinned_visible_columns();
        let left_visible = left.into_iter().map(|c| c.id.clone()).collect::<Vec<_>>();
        let center_visible = center.into_iter().map(|c| c.id.clone()).collect::<Vec<_>>();
        let right_visible = right.into_iter().map(|c| c.id.clone()).collect::<Vec<_>>();

        let header_groups = self.header_groups();
        let left_header_groups = self.left_header_groups();
        let center_header_groups = self.center_header_groups();
        let right_header_groups = self.right_header_groups();

        let core = snapshot_row_model_ids(self.core_row_model());
        let row_model = snapshot_row_model_ids(self.row_model());

        let mut cells: std::collections::BTreeMap<Arc<str>, super::RowCellsSnapshot> =
            std::collections::BTreeMap::new();
        for &row_i in self.row_model().root_rows() {
            let Some(row) = self.row_model().row(row_i) else {
                continue;
            };
            let row_id = Arc::<str>::from(row.key.0.to_string());
            if let Some(snapshot) = self.row_cells(row.key) {
                cells.insert(row_id, snapshot);
            }
        }

        super::CoreModelSnapshot {
            column_tree,
            leaf_columns: super::LeafColumnsSnapshot {
                all: all_leaf,
                visible,
                left_visible,
                center_visible,
                right_visible,
            },
            header_groups,
            left_header_groups,
            center_header_groups,
            right_header_groups,
            rows: super::CoreRowsSnapshot { core, row_model },
            cells,
        }
    }

    pub fn pinned_visible_columns(
        &self,
    ) -> (
        Vec<&super::ColumnDef<TData>>,
        Vec<&super::ColumnDef<TData>>,
        Vec<&super::ColumnDef<TData>>,
    ) {
        let visible = self.visible_columns();
        super::split_pinned_columns(visible.as_slice(), &self.state.column_pinning)
    }

    pub fn column_size(&self, id: &str) -> Option<f32> {
        let col = self.column(id)?;
        Some(super::resolved_column_size(&self.state.column_sizing, col))
    }

    pub fn column_sizing(&self) -> &super::ColumnSizingState {
        &self.state.column_sizing
    }

    pub fn column_sizing_info(&self) -> &super::ColumnSizingInfoState {
        &self.state.column_sizing_info
    }

    pub fn column_can_resize(&self, id: &str) -> Option<bool> {
        let col = self.column(id)?;
        Some(super::column_can_resize(self.options, col))
    }

    pub fn is_column_resizing(&self, id: &str) -> Option<bool> {
        let col = self.column(id)?;
        Some(
            self.state
                .column_sizing_info
                .is_resizing_column
                .as_ref()
                .is_some_and(|active| active.as_ref() == col.id.as_ref()),
        )
    }

    /// TanStack-aligned: remove an override size entry (falls back to column defaults).
    pub fn reset_column_size(&self, id: &str) -> Option<super::ColumnSizingState> {
        let col = self.column(id)?;
        let mut next = self.state.column_sizing.clone();
        next.remove(&col.id);
        Some(next)
    }

    pub fn started_column_resize(
        &self,
        id: &str,
        pointer_x: f32,
    ) -> Option<super::ColumnSizingInfoState> {
        let col = self.column(id)?;
        if !super::column_can_resize(self.options, col) {
            return Some(self.state.column_sizing_info.clone());
        }

        let start = super::resolved_column_size(&self.state.column_sizing, col);
        let mut next = self.state.column_sizing_info.clone();
        super::begin_column_resize(
            &mut next,
            col.id.clone(),
            pointer_x,
            vec![(col.id.clone(), start)],
        );
        Some(next)
    }

    pub fn dragged_column_resize(
        &self,
        pointer_x: f32,
    ) -> (super::ColumnSizingState, super::ColumnSizingInfoState) {
        let mut sizing = self.state.column_sizing.clone();
        let mut info = self.state.column_sizing_info.clone();
        super::drag_column_resize(
            self.options.column_resize_mode,
            self.options.column_resize_direction,
            &mut sizing,
            &mut info,
            pointer_x,
        );
        (sizing, info)
    }

    pub fn ended_column_resize(
        &self,
        pointer_x: Option<f32>,
    ) -> (super::ColumnSizingState, super::ColumnSizingInfoState) {
        let mut sizing = self.state.column_sizing.clone();
        let mut info = self.state.column_sizing_info.clone();
        super::end_column_resize(
            self.options.column_resize_mode,
            self.options.column_resize_direction,
            &mut sizing,
            &mut info,
            pointer_x,
        );
        (sizing, info)
    }

    pub fn total_size(&self) -> f32 {
        self.visible_columns()
            .into_iter()
            .map(|c| super::resolved_column_size(&self.state.column_sizing, c))
            .sum()
    }

    pub fn pinned_total_sizes(&self) -> (f32, f32, f32) {
        let (left, center, right) = self.pinned_visible_columns();
        let sizing = &self.state.column_sizing;

        let left = left
            .into_iter()
            .map(|c| super::resolved_column_size(sizing, c))
            .sum();
        let center = center
            .into_iter()
            .map(|c| super::resolved_column_size(sizing, c))
            .sum();
        let right = right
            .into_iter()
            .map(|c| super::resolved_column_size(sizing, c))
            .sum();

        (left, center, right)
    }

    pub fn left_total_size(&self) -> f32 {
        self.pinned_total_sizes().0
    }

    pub fn center_total_size(&self) -> f32 {
        self.pinned_total_sizes().1
    }

    pub fn right_total_size(&self) -> f32 {
        self.pinned_total_sizes().2
    }

    /// TanStack-aligned: return the start offset (x) for a column within a pinned region.
    pub fn column_start(&self, column_id: &str, region: super::ColumnSizingRegion) -> Option<f32> {
        let col = self.column(column_id)?;
        let sizing = &self.state.column_sizing;

        let (left, center, right) = self.pinned_visible_columns();
        let mut offset = 0.0;

        match region {
            super::ColumnSizingRegion::All => {
                for c in left.into_iter().chain(center).chain(right) {
                    if c.id.as_ref() == col.id.as_ref() {
                        return Some(offset);
                    }
                    offset += super::resolved_column_size(sizing, c);
                }
            }
            super::ColumnSizingRegion::Left => {
                for c in left {
                    if c.id.as_ref() == col.id.as_ref() {
                        return Some(offset);
                    }
                    offset += super::resolved_column_size(sizing, c);
                }
            }
            super::ColumnSizingRegion::Center => {
                for c in center {
                    if c.id.as_ref() == col.id.as_ref() {
                        return Some(offset);
                    }
                    offset += super::resolved_column_size(sizing, c);
                }
            }
            super::ColumnSizingRegion::Right => {
                for c in right {
                    if c.id.as_ref() == col.id.as_ref() {
                        return Some(offset);
                    }
                    offset += super::resolved_column_size(sizing, c);
                }
            }
        }
        None
    }

    /// TanStack-aligned: return the after offset (remaining width) for a column within a pinned region.
    pub fn column_after(&self, column_id: &str, region: super::ColumnSizingRegion) -> Option<f32> {
        let col = self.column(column_id)?;
        let sizing = &self.state.column_sizing;

        let (left, center, right) = self.pinned_visible_columns();

        fn after_for<'a, TData: 'a>(
            cols: impl IntoIterator<Item = &'a super::ColumnDef<TData>>,
            target: &super::ColumnDef<TData>,
            sizing: &super::ColumnSizingState,
        ) -> Option<f32> {
            let cols: Vec<&super::ColumnDef<TData>> = cols.into_iter().collect();
            let index = cols
                .iter()
                .position(|c| c.id.as_ref() == target.id.as_ref())?;
            let mut sum = 0.0;
            for c in cols.into_iter().skip(index + 1) {
                sum += super::resolved_column_size(sizing, c);
            }
            Some(sum)
        }

        match region {
            super::ColumnSizingRegion::All => {
                after_for(left.into_iter().chain(center).chain(right), col, sizing)
            }
            super::ColumnSizingRegion::Left => after_for(left, col, sizing),
            super::ColumnSizingRegion::Center => after_for(center, col, sizing),
            super::ColumnSizingRegion::Right => after_for(right, col, sizing),
        }
    }

    pub fn core_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model.get_or_init(|| {
            build_core_row_model(self.data, &*self.get_row_key, self.get_sub_rows.as_deref())
        })
    }

    pub fn pre_filtered_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model()
    }

    pub fn filtered_row_model(&self) -> &RowModel<'a, TData> {
        if self.options.manual_filtering {
            return self.pre_filtered_row_model();
        }
        self.filtered_row_model.get_or_init(|| {
            super::filter_row_model(
                self.pre_filtered_row_model(),
                &self.columns,
                &self.state.column_filters,
                self.state.global_filter.clone(),
                self.options,
                &self.filter_fns,
                &self.global_filter_fn,
                self.get_column_can_global_filter.as_deref(),
            )
        })
    }

    pub fn pre_sorted_row_model(&self) -> &RowModel<'a, TData> {
        self.filtered_row_model()
    }

    pub fn sorted_row_model(&self) -> &RowModel<'a, TData> {
        if self.options.manual_sorting {
            return self.pre_sorted_row_model();
        }
        self.sorted_row_model.get_or_init(|| {
            super::sort_row_model(
                self.pre_sorted_row_model(),
                &self.columns,
                &self.state.sorting,
                &self.sorting_fns,
            )
        })
    }

    pub fn pre_pagination_row_model(&self) -> &RowModel<'a, TData> {
        if self.options.paginate_expanded_rows {
            self.expanded_row_model()
        } else {
            self.sorted_row_model()
        }
    }

    pub fn pre_expanded_row_model(&self) -> &RowModel<'a, TData> {
        self.sorted_row_model()
    }

    pub fn expanded_row_model(&self) -> &RowModel<'a, TData> {
        if !self.options.paginate_expanded_rows {
            return self.pre_expanded_row_model();
        }
        if self.options.manual_expanding {
            return self.pre_expanded_row_model();
        }
        if !super::is_some_rows_expanded(&self.state.expanding) {
            return self.pre_expanded_row_model();
        }
        self.expanded_row_model.get_or_init(|| {
            super::expand_row_model(self.pre_expanded_row_model(), &self.state.expanding)
        })
    }

    pub fn row_model(&self) -> &RowModel<'a, TData> {
        if self.options.manual_pagination {
            return self.pre_pagination_row_model();
        }
        if self.options.paginate_expanded_rows {
            return self.paginated_row_model.get_or_init(|| {
                super::paginate_row_model(self.pre_pagination_row_model(), self.state.pagination)
            });
        }

        let paginated = self.paginated_row_model.get_or_init(|| {
            super::paginate_row_model(self.pre_pagination_row_model(), self.state.pagination)
        });
        self.expanded_paginated_row_model.get_or_init(|| {
            let mut out = super::expand_row_model(paginated, &self.state.expanding);
            rebuild_flat_rows_from_roots_including_duplicates(&mut out);
            out
        })
    }

    pub fn pre_selected_row_model(&self) -> &RowModel<'a, TData> {
        self.core_row_model()
    }

    pub fn selected_row_model(&self) -> &RowModel<'a, TData> {
        self.selected_row_model.get_or_init(|| {
            super::select_rows_fn(self.pre_selected_row_model(), &self.state.row_selection)
        })
    }

    pub fn filtered_selected_row_model(&self) -> &RowModel<'a, TData> {
        self.filtered_selected_row_model.get_or_init(|| {
            if self.state.row_selection.is_empty() {
                return RowModel {
                    root_rows: Vec::new(),
                    flat_rows: Vec::new(),
                    rows_by_key: HashMap::new(),
                    arena: Vec::new(),
                };
            }
            super::select_rows_fn(self.filtered_row_model(), &self.state.row_selection)
        })
    }

    pub fn grouped_selected_row_model(&self) -> &RowModel<'a, TData> {
        self.grouped_selected_row_model.get_or_init(|| {
            if self.state.row_selection.is_empty() {
                return RowModel {
                    root_rows: Vec::new(),
                    flat_rows: Vec::new(),
                    rows_by_key: HashMap::new(),
                    arena: Vec::new(),
                };
            }
            super::select_rows_fn(self.sorted_row_model(), &self.state.row_selection)
        })
    }

    pub fn page_selected_row_model(&self) -> &RowModel<'a, TData> {
        self.page_selected_row_model.get_or_init(|| {
            if self.state.row_selection.is_empty() {
                return RowModel {
                    root_rows: Vec::new(),
                    flat_rows: Vec::new(),
                    rows_by_key: HashMap::new(),
                    arena: Vec::new(),
                };
            }
            super::select_rows_fn(self.row_model(), &self.state.row_selection)
        })
    }

    pub fn row_is_selected(&self, row_key: RowKey) -> bool {
        super::is_row_selected(row_key, &self.state.row_selection)
    }

    pub fn row_is_some_selected(&self, row_key: RowKey) -> bool {
        self.core_row_model().row_by_key(row_key).is_some_and(|i| {
            super::row_is_some_selected(self.core_row_model(), &self.state.row_selection, i)
        })
    }

    pub fn row_is_all_sub_rows_selected(&self, row_key: RowKey) -> bool {
        self.core_row_model().row_by_key(row_key).is_some_and(|i| {
            super::row_is_all_sub_rows_selected(self.core_row_model(), &self.state.row_selection, i)
        })
    }

    pub fn toggled_row_selected(
        &self,
        row_key: RowKey,
        value: Option<bool>,
        select_children: bool,
    ) -> super::RowSelectionState {
        super::toggle_row_selected(
            self.core_row_model(),
            &self.state.row_selection,
            row_key,
            value,
            select_children,
            self.options.enable_row_selection,
            self.options.enable_multi_row_selection,
            self.options.enable_sub_row_selection,
        )
    }

    pub fn is_all_rows_selected(&self) -> bool {
        super::is_all_rows_selected(
            self.filtered_row_model(),
            &self.state.row_selection,
            self.options.enable_row_selection,
        )
    }

    pub fn is_some_rows_selected(&self) -> bool {
        super::is_some_rows_selected(self.filtered_row_model(), &self.state.row_selection)
    }

    pub fn is_all_page_rows_selected(&self) -> bool {
        super::is_all_page_rows_selected(
            self.row_model(),
            &self.state.row_selection,
            self.options.enable_row_selection,
        )
    }

    pub fn is_some_page_rows_selected(&self) -> bool {
        super::is_some_page_rows_selected(
            self.row_model(),
            &self.state.row_selection,
            self.options.enable_row_selection,
        )
    }

    pub fn filtered_row_count(&self) -> usize {
        self.filtered_row_model().root_rows().len()
    }

    pub fn filtered_flat_row_count(&self) -> usize {
        self.filtered_row_model().flat_rows().len()
    }

    pub fn filtered_selected_row_count(&self) -> usize {
        super::selected_root_row_count(self.filtered_row_model(), &self.state.row_selection)
    }

    pub fn filtered_selected_flat_row_count(&self) -> usize {
        super::selected_flat_row_count(self.filtered_row_model(), &self.state.row_selection)
    }

    pub fn toggled_all_rows_selected(&self, value: Option<bool>) -> super::RowSelectionState {
        super::toggle_all_rows_selected(
            self.filtered_row_model(),
            &self.state.row_selection,
            value,
            self.options.enable_row_selection,
        )
    }

    pub fn toggled_all_page_rows_selected(&self, value: Option<bool>) -> super::RowSelectionState {
        super::toggle_all_page_rows_selected(
            self.row_model(),
            &self.state.row_selection,
            value,
            self.options.enable_row_selection,
        )
    }

    pub fn faceted_row_model(&self, column_id: &str) -> Option<&RowModel<'a, TData>> {
        let column_index = self.column_index(column_id)?;

        if self.options.manual_filtering {
            return Some(self.pre_filtered_row_model());
        }

        let caches = self.faceted_row_model_by_column.get_or_init(|| {
            let mut v = Vec::with_capacity(self.columns.len());
            for _ in 0..self.columns.len() {
                v.push(OnceCell::new());
            }
            v
        });

        Some(caches[column_index].get_or_init(|| {
            super::faceted_row_model_excluding(
                self.pre_filtered_row_model(),
                &self.columns,
                &self.state.column_filters,
                self.state.global_filter.clone(),
                self.options,
                &self.filter_fns,
                &self.global_filter_fn,
                self.get_column_can_global_filter.as_deref(),
                Some(column_id),
            )
        }))
    }

    pub fn faceted_unique_values(&self, column_id: &str) -> Option<&super::FacetCounts> {
        let column_index = self.column_index(column_id)?;
        let column = self.column(column_id)?;

        let caches = self.faceted_unique_values_by_column.get_or_init(|| {
            let mut v = Vec::with_capacity(self.columns.len());
            for _ in 0..self.columns.len() {
                v.push(OnceCell::new());
            }
            v
        });

        Some(caches[column_index].get_or_init(|| {
            let model = self
                .faceted_row_model(column_id)
                .unwrap_or_else(|| self.row_model());
            super::faceted_unique_values(model, column)
        }))
    }

    pub fn faceted_unique_value_labels(&self, column_id: &str) -> Option<&super::FacetLabels<'a>> {
        let column_index = self.column_index(column_id)?;
        let column = self.column(column_id)?;

        let caches = self.faceted_unique_labels_by_column.get_or_init(|| {
            let mut v = Vec::with_capacity(self.columns.len());
            for _ in 0..self.columns.len() {
                v.push(OnceCell::new());
            }
            v
        });

        Some(caches[column_index].get_or_init(|| {
            let model = self
                .faceted_row_model(column_id)
                .unwrap_or_else(|| self.row_model());
            super::faceted_unique_value_labels(model, column)
        }))
    }

    pub fn faceted_min_max_u64(&self, column_id: &str) -> Option<(u64, u64)> {
        let column_index = self.column_index(column_id)?;
        let column = self.column(column_id)?;

        let caches = self.faceted_min_max_u64_by_column.get_or_init(|| {
            let mut v = Vec::with_capacity(self.columns.len());
            for _ in 0..self.columns.len() {
                v.push(OnceCell::new());
            }
            v
        });

        *caches[column_index].get_or_init(|| {
            let model = self
                .faceted_row_model(column_id)
                .unwrap_or_else(|| self.row_model());
            super::faceted_min_max_u64(model, column)
        })
    }
}

fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn default_row_key_for_index_path<TData>(
    _: &TData,
    index: usize,
    parent: Option<&RowKey>,
) -> RowKey {
    if let Some(parent) = parent {
        // Mix parent and child index in an order-sensitive way (avoid trivial collisions like
        // `(parent=0, i=1)` vs `(parent=1, i=0)` that happen with XOR).
        let z = parent
            .0
            .wrapping_mul(0x9e37_79b9_7f4a_7c15)
            .wrapping_add((index as u64).wrapping_add(0xbf58_476d_1ce4_e5b9));
        RowKey(splitmix64(z))
    } else {
        RowKey::from_index(index)
    }
}

fn build_core_row_model<'a, TData>(
    data: &'a [TData],
    get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
    get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
) -> RowModel<'a, TData> {
    let mut root_rows: Vec<RowIndex> = Vec::new();
    let mut flat_rows: Vec<RowIndex> = Vec::new();
    let mut rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut arena: Vec<Row<'a, TData>> = Vec::new();

    fn access_rows<'a, TData>(
        original_rows: &'a [TData],
        depth: u16,
        parent: Option<RowIndex>,
        parent_key: Option<&RowKey>,
        get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
        get_sub_rows: Option<&dyn for<'r> Fn(&'r TData, usize) -> Option<&'r [TData]>>,
        root_out: &mut Vec<RowIndex>,
        flat_out: &mut Vec<RowIndex>,
        rows_by_key: &mut HashMap<RowKey, RowIndex>,
        arena: &mut Vec<Row<'a, TData>>,
    ) -> Vec<RowIndex> {
        let mut rows: Vec<RowIndex> = Vec::with_capacity(original_rows.len());
        for (index, original) in original_rows.iter().enumerate() {
            let key = get_row_key(original, index, parent_key);
            let row_index = arena.len();
            arena.push(Row {
                key,
                original,
                index,
                depth,
                parent,
                parent_key: parent_key.copied(),
                sub_rows: Vec::new(),
            });
            flat_out.push(row_index);
            rows_by_key.insert(key, row_index);
            rows.push(row_index);

            if let Some(get_sub_rows) = get_sub_rows
                && let Some(sub) = get_sub_rows(original, index)
                && !sub.is_empty()
            {
                let children = access_rows(
                    sub,
                    depth.saturating_add(1),
                    Some(row_index),
                    Some(&key),
                    get_row_key,
                    Some(get_sub_rows),
                    root_out,
                    flat_out,
                    rows_by_key,
                    arena,
                );
                if let Some(row) = arena.get_mut(row_index) {
                    row.sub_rows = children;
                }
            }
        }

        if depth == 0 {
            root_out.extend_from_slice(&rows);
        }

        rows
    }

    let _ = access_rows(
        data,
        0,
        None,
        None,
        get_row_key,
        get_sub_rows,
        &mut root_rows,
        &mut flat_rows,
        &mut rows_by_key,
        &mut arena,
    );

    RowModel {
        root_rows,
        flat_rows,
        rows_by_key,
        arena,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::is_column_visible;
    use crate::table::{
        ColumnDef, ColumnFilter, ColumnId, ColumnPinPosition, ColumnSizingRegion, PaginationState,
        SortSpec, TableOptions, TableState, create_column_helper,
    };
    use std::sync::Arc;

    #[derive(Debug, Clone)]
    struct Person {
        #[allow(dead_code)]
        name: String,
        sub_rows: Option<Vec<Person>>,
    }

    fn make_people(rows: usize, sub_rows: usize) -> Vec<Person> {
        (0..rows)
            .map(|i| Person {
                name: format!("Person {i}"),
                sub_rows: (sub_rows > 0).then(|| {
                    (0..sub_rows)
                        .map(|j| Person {
                            name: format!("Person {i}.{j}"),
                            sub_rows: None,
                        })
                        .collect()
                }),
            })
            .collect()
    }

    #[test]
    fn core_row_model_produces_flat_rows_and_key_map() {
        let data = make_people(3, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3);
        assert!(model.row_by_key(RowKey::from_index(0)).is_some());
        assert!(model.row_by_key(RowKey::from_index(1)).is_some());
        assert!(model.row_by_key(RowKey::from_index(2)).is_some());
    }

    #[test]
    fn core_row_model_recurses_into_sub_rows_and_assigns_unique_keys() {
        let data = make_people(3, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        assert_eq!(model.root_rows().len(), 3);
        assert_eq!(model.flat_rows().len(), 3 + 3 * 2);

        let root_0 = model.row(model.root_rows()[0]).expect("root row 0");
        assert_eq!(root_0.sub_rows.len(), 2);

        let c0 = model.row(root_0.sub_rows[0]).expect("root 0 child 0").key;
        let c1 = model.row(root_0.sub_rows[1]).expect("root 0 child 1").key;
        assert_ne!(c0, c1);
        assert_ne!(c0, root_0.key);
        assert_ne!(c1, root_0.key);
        assert!(model.row_by_key(c0).is_some());
        assert!(model.row_by_key(c1).is_some());
    }

    #[test]
    fn core_row_model_allows_custom_stable_row_keys() {
        let data = make_people(2, 0);
        let table = Table::builder(&data)
            .get_row_key(|_p, i, _parent| RowKey(10_000 + i as u64))
            .build();
        let model = table.core_row_model();

        assert!(model.row_by_key(RowKey(10_000)).is_some());
        assert!(model.row_by_key(RowKey(10_001)).is_some());
        assert!(model.row_by_key(RowKey::from_index(0)).is_none());
    }

    #[derive(Debug, Clone)]
    struct Item {
        value: i32,
    }

    #[test]
    fn table_sorted_row_model_uses_state_sorting() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let table = Table::builder(&data)
            .columns(columns)
            .state(TableState {
                sorting: vec![SortSpec {
                    column: "value".into(),
                    desc: false,
                }],
                ..TableState::default()
            })
            .build();

        let sorted = table.sorted_row_model();
        let keys = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec![1, 0, 2]);
        assert!(std::ptr::eq(sorted, table.sorted_row_model()));
    }

    #[test]
    fn table_row_model_applies_pagination_after_sorting() {
        let data = vec![
            Item { value: 2 },
            Item { value: 1 },
            Item { value: 3 },
            Item { value: 0 },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let table = Table::builder(&data)
            .columns(columns)
            .state(TableState {
                sorting: vec![SortSpec {
                    column: "value".into(),
                    desc: false,
                }],
                pagination: PaginationState {
                    page_index: 0,
                    page_size: 2,
                },
                ..TableState::default()
            })
            .build();

        let paged = table.row_model();
        let keys = paged
            .root_rows()
            .iter()
            .filter_map(|&i| paged.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec![3, 1]);
        assert!(std::ptr::eq(paged, table.row_model()));
    }

    #[test]
    fn table_selected_row_model_uses_state_row_selection() {
        let data = make_people(3, 0);
        let table = Table::builder(&data)
            .state(TableState {
                row_selection: [RowKey(1)].into_iter().collect(),
                ..TableState::default()
            })
            .build();

        let selected = table.selected_row_model();
        assert_eq!(selected.root_rows().len(), 1);
        assert!(selected.row_by_key(RowKey(1)).is_some());
        assert!(std::ptr::eq(selected, table.selected_row_model()));
    }

    #[test]
    fn table_visible_columns_respects_order_then_visibility() {
        let data = vec![Item { value: 1 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper.clone().accessor("a", |it| it.value),
            helper.clone().accessor("b", |it| it.value),
            helper.accessor("c", |it| it.value),
        ];

        let mut state = TableState::default();
        state.column_order = vec!["c".into(), "a".into()];
        state.column_visibility.insert("b".into(), false);

        let table = Table::builder(&data).columns(columns).state(state).build();
        let visible = table.visible_columns();
        let ids = visible.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>();

        assert_eq!(ids, vec!["c", "a"]);
    }

    #[test]
    fn table_column_size_reads_from_state() {
        let data = vec![Item { value: 1 }];

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let mut state = TableState::default();
        state.column_sizing.insert("value".into(), 120.0);

        let table = Table::builder(&data).columns(columns).state(state).build();
        assert_eq!(table.column_size("value"), Some(120.0));
        assert_eq!(table.column_size("missing"), None);
    }

    #[derive(Debug, Clone)]
    struct TreeNode {
        id: u64,
        children: Vec<TreeNode>,
    }

    #[test]
    fn expanding_default_is_collapsed_and_does_not_allocate() {
        let data = vec![
            TreeNode {
                id: 1,
                children: vec![
                    TreeNode {
                        id: 10,
                        children: Vec::new(),
                    },
                    TreeNode {
                        id: 11,
                        children: Vec::new(),
                    },
                ],
            },
            TreeNode {
                id: 2,
                children: vec![TreeNode {
                    id: 20,
                    children: Vec::new(),
                }],
            },
        ];

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _parent| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .build();

        let pre = table.pre_expanded_row_model();
        let expanded = table.expanded_row_model();
        assert!(
            std::ptr::eq(pre, expanded),
            "empty expanding should not allocate"
        );
        assert_eq!(expanded.root_rows().len(), 2, "collapsed shows only roots");
    }

    #[test]
    fn expanding_flattens_visible_rows_under_expanded_parents() {
        let data = vec![
            TreeNode {
                id: 1,
                children: vec![
                    TreeNode {
                        id: 10,
                        children: Vec::new(),
                    },
                    TreeNode {
                        id: 11,
                        children: Vec::new(),
                    },
                ],
            },
            TreeNode {
                id: 2,
                children: vec![TreeNode {
                    id: 20,
                    children: Vec::new(),
                }],
            },
        ];

        let mut state = TableState::default();
        state.expanding = [RowKey(1)].into_iter().collect();

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _parent| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(state)
            .build();

        let expanded = table.expanded_row_model();
        let keys = expanded
            .root_rows()
            .iter()
            .filter_map(|&i| expanded.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(keys, vec![1, 10, 11, 2]);
    }

    #[test]
    fn paginate_expanded_rows_true_counts_children_in_pages() {
        let data = vec![
            TreeNode {
                id: 1,
                children: vec![
                    TreeNode {
                        id: 10,
                        children: Vec::new(),
                    },
                    TreeNode {
                        id: 11,
                        children: Vec::new(),
                    },
                ],
            },
            TreeNode {
                id: 2,
                children: vec![TreeNode {
                    id: 20,
                    children: Vec::new(),
                }],
            },
        ];

        let mut state = TableState::default();
        state.expanding = [RowKey(1)].into_iter().collect();
        state.pagination = PaginationState {
            page_index: 0,
            page_size: 2,
        };

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _parent| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(state)
            .build();

        let model = table.row_model();
        let keys = model
            .root_rows()
            .iter()
            .filter_map(|&i| model.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();
        assert_eq!(keys, vec![1, 10]);
    }

    #[test]
    fn paginate_expanded_rows_false_expands_within_parent_page() {
        let data = vec![
            TreeNode {
                id: 1,
                children: vec![
                    TreeNode {
                        id: 10,
                        children: Vec::new(),
                    },
                    TreeNode {
                        id: 11,
                        children: Vec::new(),
                    },
                ],
            },
            TreeNode {
                id: 2,
                children: vec![TreeNode {
                    id: 20,
                    children: Vec::new(),
                }],
            },
        ];

        let mut state = TableState::default();
        state.expanding = [RowKey(1)].into_iter().collect();
        state.pagination = PaginationState {
            page_index: 0,
            page_size: 1,
        };

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _parent| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(state)
            .options(TableOptions {
                paginate_expanded_rows: false,
                ..Default::default()
            })
            .build();

        let model = table.row_model();
        let keys = model
            .root_rows()
            .iter()
            .filter_map(|&i| model.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();
        assert_eq!(keys, vec![1, 10, 11]);
    }

    #[test]
    fn manual_filtering_skips_filtered_row_model() {
        #[derive(Debug, Clone)]
        struct Item {
            label: Arc<str>,
        }

        let data = vec![
            Item {
                label: Arc::from("a"),
            },
            Item {
                label: Arc::from("b"),
            },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper
                .accessor("label", |it| it.label.clone())
                .filter_by(|row, q| row.label.as_ref() == q),
        ];

        let mut state = TableState::default();
        state.global_filter = Some(serde_json::Value::String("b".to_string()));

        let table = Table::builder(&data)
            .columns(columns)
            .state(state)
            .options(TableOptions {
                manual_filtering: true,
                ..Default::default()
            })
            .build();

        assert!(std::ptr::eq(
            table.filtered_row_model(),
            table.core_row_model()
        ));
    }

    #[test]
    fn manual_sorting_skips_sorted_row_model() {
        #[derive(Debug, Clone)]
        struct Item {
            value: i32,
        }

        let data = vec![Item { value: 2 }, Item { value: 1 }];
        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];

        let mut state = TableState::default();
        state.sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let table = Table::builder(&data)
            .columns(columns)
            .state(state)
            .options(TableOptions {
                manual_sorting: true,
                ..Default::default()
            })
            .build();

        assert!(std::ptr::eq(
            table.sorted_row_model(),
            table.pre_sorted_row_model()
        ));
    }

    #[test]
    fn manual_pagination_skips_row_model() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: i32,
        }

        let data = (0..20).map(|i| Item { value: i }).collect::<Vec<_>>();
        let mut state = TableState::default();
        state.pagination = PaginationState {
            page_index: 1,
            page_size: 5,
        };

        let table = Table::builder(&data)
            .state(state)
            .options(TableOptions {
                manual_pagination: true,
                ..Default::default()
            })
            .build();

        assert!(std::ptr::eq(
            table.row_model(),
            table.pre_pagination_row_model()
        ));
    }

    #[test]
    fn table_expanding_all_marks_all_rows_expanded() {
        let data = vec![TreeNode {
            id: 1,
            children: vec![TreeNode {
                id: 10,
                children: Vec::new(),
            }],
        }];

        let mut state = TableState::default();
        state.expanding = crate::table::ExpandingState::All;

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _parent| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(state)
            .build();

        assert!(table.is_some_rows_expanded());
        assert!(table.is_all_rows_expanded());
        assert!(table.row_can_expand(RowKey(1)));
        assert!(table.row_is_all_parents_expanded(RowKey(10)));
    }

    #[test]
    fn keep_pinned_rows_true_keeps_pins_across_pagination() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = (0..20).map(|i| Item { value: i }).collect::<Vec<_>>();

        let mut state = TableState::default();
        state.pagination = PaginationState {
            page_index: 1,
            page_size: 5,
        };
        state.row_pinning.top = vec![RowKey(1)];

        let table_keep = Table::builder(&data)
            .state(state.clone())
            .options(TableOptions {
                keep_pinned_rows: true,
                ..Default::default()
            })
            .build();
        assert_eq!(table_keep.top_row_keys(), vec![RowKey(1)]);

        let table_no_keep = Table::builder(&data)
            .state(state)
            .options(TableOptions {
                keep_pinned_rows: false,
                ..Default::default()
            })
            .build();
        assert!(table_no_keep.top_row_keys().is_empty());
    }

    #[test]
    fn keep_pinned_rows_true_respects_expanded_parents() {
        let data = vec![TreeNode {
            id: 1,
            children: vec![TreeNode {
                id: 10,
                children: Vec::new(),
            }],
        }];

        let mut collapsed = TableState::default();
        collapsed.row_pinning.top = vec![RowKey(10)];

        let table_collapsed = Table::builder(&data)
            .get_row_key(|n, _i, _p| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(collapsed)
            .options(TableOptions {
                keep_pinned_rows: true,
                ..Default::default()
            })
            .build();
        assert!(table_collapsed.top_row_keys().is_empty());

        let mut expanded = TableState::default();
        expanded.expanding = [RowKey(1)].into_iter().collect();
        expanded.row_pinning.top = vec![RowKey(10)];

        let table_expanded = Table::builder(&data)
            .get_row_key(|n, _i, _p| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(expanded)
            .options(TableOptions {
                keep_pinned_rows: true,
                ..Default::default()
            })
            .build();
        assert_eq!(table_expanded.top_row_keys(), vec![RowKey(10)]);
    }

    #[test]
    fn center_row_keys_excludes_pinned_rows() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = (0..5).map(|i| Item { value: i }).collect::<Vec<_>>();
        let mut state = TableState::default();
        state.row_pinning.top = vec![RowKey(0)];
        state.row_pinning.bottom = vec![RowKey(4)];

        let table = Table::builder(&data).state(state).build();
        assert_eq!(
            table.center_row_keys(),
            vec![RowKey(1), RowKey(2), RowKey(3)]
        );
    }

    #[test]
    fn table_faceting_excludes_own_filter_and_can_return_labels() {
        #[derive(Debug, Clone)]
        struct Item {
            status_key: u64,
            status_label: Arc<str>,
            role_key: u64,
            role_label: Arc<str>,
        }

        let data = vec![
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
        ];

        let status = ColumnDef::new("status")
            .filter_by(|it: &Item, q| it.status_label.as_ref() == q)
            .facet_key_by(|it: &Item| it.status_key)
            .facet_str_by(|it: &Item| it.status_label.as_ref());
        let role = ColumnDef::new("role")
            .filter_by(|it: &Item, q| it.role_label.as_ref() == q)
            .facet_key_by(|it: &Item| it.role_key)
            .facet_str_by(|it: &Item| it.role_label.as_ref());

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

        let table = Table::builder(&data)
            .columns(vec![status, role])
            .state(state)
            .build();

        let counts = table.faceted_unique_values("status").unwrap();
        assert_eq!(counts.get(&1).copied(), Some(1));
        assert_eq!(counts.get(&2).copied(), Some(1));

        let labels = table.faceted_unique_value_labels("status").unwrap();
        assert_eq!(labels.get(&1).copied(), Some("A"));
        assert_eq!(labels.get(&2).copied(), Some("B"));

        assert_eq!(table.faceted_min_max_u64("status"), Some((1, 2)));
    }

    #[test]
    fn table_row_selection_page_toggle_and_indeterminate_queries() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = (0..10).map(|i| Item { value: i }).collect::<Vec<_>>();
        let mut state = TableState::default();
        state.pagination = PaginationState {
            page_index: 0,
            page_size: 3,
        };

        let table = Table::builder(&data).state(state).build();
        assert!(!table.is_all_page_rows_selected());
        assert!(!table.is_some_page_rows_selected());

        let selection = table.toggled_all_page_rows_selected(None);
        let table2 = Table::builder(&data)
            .state(TableState {
                pagination: table.state().pagination,
                row_selection: selection,
                ..TableState::default()
            })
            .build();
        assert!(table2.is_all_page_rows_selected());
        assert!(!table2.is_some_page_rows_selected());

        let selection = table2.toggled_all_page_rows_selected(None);
        assert!(selection.is_empty());
    }

    #[test]
    fn table_row_selection_filtered_selected_counts_match_filtered_rows() {
        #[derive(Debug, Clone)]
        struct Item {
            status: Arc<str>,
        }

        let data = vec![
            Item { status: "A".into() },
            Item { status: "B".into() },
            Item { status: "A".into() },
        ];

        let status = ColumnDef::new("status").filter_by(|it: &Item, q| it.status.as_ref() == q);

        let mut state = TableState::default();
        state.column_filters = vec![ColumnFilter {
            column: "status".into(),
            value: serde_json::Value::from("A"),
        }];
        state.row_selection = [RowKey::from_index(0)].into_iter().collect();

        let table = Table::builder(&data)
            .columns(vec![status])
            .state(state)
            .build();

        assert_eq!(table.filtered_row_count(), 2);
        assert_eq!(table.filtered_selected_row_count(), 1);
        assert_eq!(table.filtered_selected_flat_row_count(), 1);
        assert!(table.is_some_rows_selected());
        assert!(!table.is_all_rows_selected());

        let selection = table.toggled_all_rows_selected(Some(true));
        let table2 = Table::builder(&data)
            .columns(vec![
                ColumnDef::new("status").filter_by(|it: &Item, q| it.status.as_ref() == q),
            ])
            .state(TableState {
                column_filters: table.state().column_filters.clone(),
                row_selection: selection,
                ..TableState::default()
            })
            .build();

        assert!(table2.is_all_rows_selected());
        assert!(!table2.is_some_rows_selected());
    }

    #[test]
    fn table_filtered_selected_row_model_intersects_filtered_rows() {
        #[derive(Debug, Clone)]
        struct Item {
            status: Arc<str>,
        }

        let data = vec![
            Item { status: "A".into() },
            Item { status: "B".into() },
            Item { status: "A".into() },
        ];

        let status = ColumnDef::new("status").filter_by(|it: &Item, q| it.status.as_ref() == q);

        let mut state = TableState::default();
        state.column_filters = vec![ColumnFilter {
            column: "status".into(),
            value: serde_json::Value::from("A"),
        }];
        state.row_selection = [RowKey::from_index(0), RowKey::from_index(1)]
            .into_iter()
            .collect();

        let table = Table::builder(&data)
            .columns(vec![status])
            .state(state)
            .build();

        let selected = table.filtered_selected_row_model();
        assert_eq!(selected.root_rows().len(), 1);
        assert!(selected.row_by_key(RowKey::from_index(0)).is_some());
        assert!(std::ptr::eq(selected, table.filtered_selected_row_model()));
    }

    #[test]
    fn table_page_selected_row_model_only_includes_page_rows() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = (0..10).map(|i| Item { value: i }).collect::<Vec<_>>();
        let mut state = TableState::default();
        state.pagination = PaginationState {
            page_index: 1,
            page_size: 3,
        };
        state.row_selection = [RowKey::from_index(0), RowKey::from_index(4)]
            .into_iter()
            .collect();

        let table = Table::builder(&data).state(state).build();

        let selected = table.page_selected_row_model();
        assert_eq!(selected.root_rows().len(), 1);
        assert!(selected.row_by_key(RowKey::from_index(4)).is_some());
        assert!(std::ptr::eq(selected, table.page_selected_row_model()));
    }

    #[test]
    fn table_column_visibility_toggle_respects_enable_hiding() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = vec![Item { value: 1 }];
        let columns = vec![
            ColumnDef::new("a").enable_hiding(true),
            ColumnDef::new("b").enable_hiding(false),
        ];

        let table = Table::builder(&data).columns(columns).build();
        assert_eq!(table.column_can_hide("a"), Some(true));
        assert_eq!(table.column_can_hide("b"), Some(false));
        assert_eq!(table.is_column_visible("a"), Some(true));

        let next = table.toggled_column_visibility("a", Some(false)).unwrap();
        assert!(!is_column_visible(&next, &ColumnId::from("a")));

        let next_b = table.toggled_column_visibility("b", Some(false)).unwrap();
        assert!(is_column_visible(&next_b, &ColumnId::from("b")));
    }

    #[test]
    fn table_toggle_all_columns_visible_keeps_non_hideable_visible_when_hiding_all() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = vec![Item { value: 1 }];
        let columns = vec![
            ColumnDef::new("a").enable_hiding(true),
            ColumnDef::new("b").enable_hiding(false),
        ];

        let table = Table::builder(&data).columns(columns).build();
        let next = table.toggled_all_columns_visible(Some(false));

        assert!(!is_column_visible(&next, &ColumnId::from("a")));
        assert!(is_column_visible(&next, &ColumnId::from("b")));
    }

    #[test]
    fn table_column_order_move_respects_enable_column_ordering() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = vec![Item { value: 1 }];
        let columns = vec![
            ColumnDef::new("a").enable_ordering(true),
            ColumnDef::new("b").enable_ordering(false),
            ColumnDef::new("c").enable_ordering(true),
        ];

        let mut state = TableState::default();
        state.column_order = vec!["a".into(), "b".into(), "c".into()];
        let table = Table::builder(&data).columns(columns).state(state).build();

        let next = table.toggled_column_order_move("a", 2).unwrap();
        assert_eq!(
            next.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["b", "c", "a"]
        );

        let next_b = table.toggled_column_order_move("b", 0).unwrap();
        assert_eq!(
            next_b.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn table_column_pinning_respects_enable_column_pinning() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = vec![Item { value: 1 }];
        let columns = vec![
            ColumnDef::new("a").enable_pinning(true),
            ColumnDef::new("b").enable_pinning(false),
        ];
        let table = Table::builder(&data).columns(columns).build();

        let next = table
            .toggled_column_pinning("a", Some(ColumnPinPosition::Left))
            .unwrap();
        assert_eq!(
            next.left.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["a"]
        );

        let next_b = table
            .toggled_column_pinning("b", Some(ColumnPinPosition::Right))
            .unwrap();
        assert!(next_b.left.is_empty());
        assert!(next_b.right.is_empty());
    }

    #[test]
    fn table_column_sizing_totals_and_start_offsets_respect_pinning_and_order() {
        #[derive(Debug, Clone)]
        struct Item {
            #[allow(dead_code)]
            value: usize,
        }

        let data = vec![Item { value: 1 }];
        let columns = vec![
            ColumnDef::new("a").size(100.0),
            ColumnDef::new("b").size(50.0),
            ColumnDef::new("c").size(25.0),
        ];

        let mut state = TableState::default();
        state.column_order = vec!["b".into(), "c".into(), "a".into()];
        state.column_pinning.left = vec!["b".into()];
        state.column_pinning.right = vec!["a".into()];

        let table = Table::builder(&data).columns(columns).state(state).build();

        assert_eq!(table.left_total_size(), 50.0);
        assert_eq!(table.center_total_size(), 25.0);
        assert_eq!(table.right_total_size(), 100.0);
        assert_eq!(table.total_size(), 175.0);

        assert_eq!(table.column_start("b", ColumnSizingRegion::All), Some(0.0));
        assert_eq!(table.column_start("c", ColumnSizingRegion::All), Some(50.0));
        assert_eq!(table.column_start("a", ColumnSizingRegion::All), Some(75.0));

        assert_eq!(
            table.column_after("b", ColumnSizingRegion::All),
            Some(125.0)
        );
        assert_eq!(
            table.column_after("c", ColumnSizingRegion::All),
            Some(100.0)
        );
        assert_eq!(table.column_after("a", ColumnSizingRegion::All), Some(0.0));

        assert_eq!(table.column_start("b", ColumnSizingRegion::Left), Some(0.0));
        assert_eq!(table.column_start("c", ColumnSizingRegion::Left), None);
        assert_eq!(
            table.column_start("c", ColumnSizingRegion::Center),
            Some(0.0)
        );
        assert_eq!(
            table.column_start("a", ColumnSizingRegion::Right),
            Some(0.0)
        );

        assert_eq!(table.column_after("b", ColumnSizingRegion::Left), Some(0.0));
        assert_eq!(table.column_after("c", ColumnSizingRegion::Left), None);
        assert_eq!(
            table.column_after("c", ColumnSizingRegion::Center),
            Some(0.0)
        );
        assert_eq!(
            table.column_after("a", ColumnSizingRegion::Right),
            Some(0.0)
        );
    }
}
