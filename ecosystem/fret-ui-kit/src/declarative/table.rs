use fret_core::{Color, Corners, CursorIcon, Edges, KeyCode, Modifiers, Px, SemanticsRole};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PointerRegionProps, PressableA11y,
    PressableProps, ScrollAxis, ScrollProps, SemanticsProps,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{
    ElementContext, Theme, UiHost, action::PressablePointerDownResult, scroll::ScrollStrategy,
};

use fret_core::time::Instant;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::declarative::stack;
use crate::{Items, Justify, LayoutRefinement, MetricRef, Size, Space};

use crate::headless::table::{
    Aggregation, ColumnDef, ColumnId, ColumnResizeDirection, ColumnResizeMode, ExpandingState,
    FlatRowOrderCache, FlatRowOrderDeps, GroupedColumnMode, GroupedRowKind, PaginationBounds,
    PaginationState, Row, RowKey, SortSpec, Table, TableState, begin_column_resize, column_size,
    compute_grouped_u64_aggregations, drag_column_resize, end_column_resize, is_column_visible,
    is_row_expanded, is_row_selected, order_column_refs_for_grouping, order_columns,
    pagination_bounds, sort_grouped_row_indices_in_place, split_pinned_columns,
};

fn resolve_table_colors(theme: &Theme) -> (Color, Color, Color, Color, Color) {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("list.background"))
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_required("card"));
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_required("border"));
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(table_bg);
    let row_hover = theme
        .color_by_key("table.row.hover")
        .or_else(|| theme.color_by_key("list.hover.background"))
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    let row_active = theme
        .color_by_key("table.row.active")
        .or_else(|| theme.color_by_key("list.active.background"))
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_required("accent"));
    (table_bg, border, header_bg, row_hover, row_active)
}

fn emphasize_border(border: Color, min_alpha: f32) -> Color {
    Color {
        a: border.a.max(min_alpha),
        ..border
    }
}

fn resolve_row_height(theme: &Theme, size: Size) -> Px {
    let base = theme
        .metric_by_key("component.table.row_height")
        .or_else(|| theme.metric_by_key("component.list.row_height"))
        .unwrap_or_else(|| size.list_row_h(theme));
    Px(base.0.max(0.0))
}

fn resolve_cell_padding_x(theme: &Theme) -> Px {
    MetricRef::space(Space::N2p5).resolve(theme)
}

fn resolve_cell_padding_y(theme: &Theme) -> Px {
    MetricRef::space(Space::N1p5).resolve(theme)
}

fn sort_for_column(sorting: &[SortSpec], id: &ColumnId) -> Option<bool> {
    sorting
        .iter()
        .find(|s| s.column.as_ref() == id.as_ref())
        .map(|s| s.desc)
}

fn next_sort_for_column(current: Option<bool>) -> Option<bool> {
    match current {
        None => Some(false),
        Some(false) => Some(true),
        Some(true) => None,
    }
}

fn apply_single_sort_toggle(state: &mut TableState, col_id: &ColumnId) {
    let current = sort_for_column(&state.sorting, col_id);
    let next = next_sort_for_column(current);
    state.sorting.clear();
    if let Some(desc) = next {
        state.sorting.push(SortSpec {
            column: col_id.clone(),
            desc,
        });
    }
    state.pagination.page_index = 0;
}

fn clamp_column_width<TData>(col: &ColumnDef<TData>, props: &TableViewProps, width: f32) -> Px {
    let min_w = col.min_size.max(props.min_column_width.0).max(0.0);
    let max_w = col.max_size.max(min_w);
    Px(width.clamp(min_w, max_w))
}

fn resolve_column_width<TData>(
    col: &ColumnDef<TData>,
    state: &TableState,
    props: &TableViewProps,
) -> Px {
    let base = column_size(&state.column_sizing, &col.id).unwrap_or(props.default_column_width.0);
    let base = clamp_column_width(col, props, base);

    base
}

#[derive(Debug, Clone)]
pub struct TableViewProps {
    pub size: Size,
    pub row_height: Option<Px>,
    pub overscan: usize,
    pub default_column_width: Px,
    pub min_column_width: Px,
    /// When `true`, clicking a sortable header updates `TableState.sorting`.
    ///
    /// This is a UI-side interaction toggle; sorting math still lives in the headless engine.
    pub enable_sorting: bool,
    pub enable_column_resizing: bool,
    pub column_resize_mode: ColumnResizeMode,
    pub column_resize_direction: ColumnResizeDirection,
    pub enable_column_grouping: bool,
    pub grouped_column_mode: GroupedColumnMode,
    pub enable_row_selection: bool,
    pub single_row_selection: bool,
    /// When `false`, the table does not render an outer border/radius frame.
    ///
    /// This is useful when embedding the table inside a higher-level component that owns the
    /// surrounding chrome (e.g. a shadcn recipe with its own border + radius).
    pub draw_frame: bool,
    /// When enabled, paints table cell backgrounds/borders in a separate layer from cell content.
    ///
    /// This is a targeted performance knob intended to reduce renderer pipeline switches for
    /// text-heavy tables by avoiding per-cell interleaving of quads and text draws.
    ///
    /// Note: this may increase UI tree complexity because it introduces an overlay layer per row.
    pub optimize_paint_order: bool,
    /// When enabled, draws only coarse column-group separators instead of per-cell vertical grid lines.
    ///
    /// This reduces quad count and renderer state churn for wide tables, but it also changes the visual
    /// semantics of the grid (column-level separators are removed).
    ///
    /// Limitations / caveats:
    ///
    /// - Default: `false`.
    /// - This intentionally trades per-column dividers for only `{left|center|right}` group dividers.
    /// - It is not a stable styling contract. Prefer keeping it disabled unless profiling shows that
    ///   per-cell dividers dominate quad count and pipeline switches for your workload.
    /// - This may be replaced by a formal style option (e.g. `TableGridLines`) or removed entirely
    ///   once a better default grid strategy exists.
    pub optimize_grid_lines: bool,
}

impl Default for TableViewProps {
    fn default() -> Self {
        Self {
            size: Size::Medium,
            row_height: None,
            overscan: 2,
            default_column_width: Px(160.0),
            min_column_width: Px(40.0),
            enable_sorting: true,
            enable_column_resizing: true,
            column_resize_mode: ColumnResizeMode::OnEnd,
            column_resize_direction: ColumnResizeDirection::Ltr,
            enable_column_grouping: true,
            grouped_column_mode: GroupedColumnMode::Reorder,
            enable_row_selection: true,
            single_row_selection: true,
            draw_frame: true,
            optimize_paint_order: false,
            optimize_grid_lines: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableViewOutput {
    /// Total row count after filters (and grouping expansion), before pagination is applied.
    pub filtered_row_count: usize,
    pub pagination: PaginationBounds,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sort_toggle_cycles_and_resets_page_index() {
        let id: ColumnId = Arc::from("col");
        let mut state = TableState::default();
        state.pagination.page_index = 3;
        assert!(state.sorting.is_empty());

        apply_single_sort_toggle(&mut state, &id);
        assert_eq!(state.pagination.page_index, 0);
        assert_eq!(state.sorting.len(), 1);
        assert_eq!(state.sorting[0].column.as_ref(), id.as_ref());
        assert!(!state.sorting[0].desc);

        state.pagination.page_index = 2;
        apply_single_sort_toggle(&mut state, &id);
        assert_eq!(state.pagination.page_index, 0);
        assert_eq!(state.sorting.len(), 1);
        assert!(state.sorting[0].desc);

        state.pagination.page_index = 1;
        apply_single_sort_toggle(&mut state, &id);
        assert_eq!(state.pagination.page_index, 0);
        assert!(state.sorting.is_empty());
    }
}

#[derive(Debug, Clone)]
enum DisplayRow {
    Leaf {
        data_index: usize,
        row_key: RowKey,
        depth: usize,
    },
    Group {
        grouping_column: ColumnId,
        row_key: RowKey,
        depth: usize,
        label: Arc<str>,
        expanded: bool,
        aggregations: Arc<[(ColumnId, Arc<str>)]>,
    },
}

impl DisplayRow {
    fn row_key(&self) -> RowKey {
        match self {
            DisplayRow::Leaf { row_key, .. } | DisplayRow::Group { row_key, .. } => *row_key,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupedBaseDeps {
    items_revision: u64,
    data_len: usize,
    columns_fingerprint: u64,
    grouping: Vec<ColumnId>,
    column_filters: crate::headless::table::ColumnFiltersState,
    global_filter: crate::headless::table::GlobalFilterState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupedDisplayDeps {
    base: GroupedBaseDeps,
    sorting: crate::headless::table::SortingState,
    expanding: ExpandingState,
    page_index: usize,
    page_size: usize,
}

#[derive(Debug, Default)]
struct GroupedDisplayCache {
    base_deps: Option<GroupedBaseDeps>,
    grouped: crate::headless::table::GroupedRowModel,
    row_index_by_key: std::collections::HashMap<RowKey, usize>,
    group_labels: std::collections::HashMap<RowKey, Arc<str>>,
    group_aggs_u64: std::collections::HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
    group_aggs_text: std::collections::HashMap<RowKey, Arc<[(ColumnId, Arc<str>)]>>,

    deps: Option<GroupedDisplayDeps>,
    page_rows: Vec<DisplayRow>,
    output: TableViewOutput,
}

fn fnv1a64_bytes(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001B3;

    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn columns_fingerprint<TData>(columns: &[ColumnDef<TData>]) -> u64 {
    let mut h = fnv1a64_bytes(b"fret.table.columns.v1");
    for c in columns {
        h ^= fnv1a64_bytes(c.id.as_bytes());
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= c.enable_grouping as u64;
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= c.facet_key_fn.is_some() as u64;
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= c.facet_str_fn.is_some() as u64;
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= c.value_u64_fn.is_some() as u64;
        h = h.wrapping_mul(0x00000100000001B3);
        h ^= match c.aggregation {
            Aggregation::None => 0,
            Aggregation::Count => 1,
            Aggregation::SumU64 => 2,
            Aggregation::MinU64 => 3,
            Aggregation::MaxU64 => 4,
            Aggregation::MeanU64 => 5,
        };
        h = h.wrapping_mul(0x00000100000001B3);
    }
    h
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableNavRowKind {
    Leaf,
    Group,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TableNavRowMeta {
    row_key: RowKey,
    kind: TableNavRowKind,
}

#[derive(Default)]
struct TableKeyboardNavState {
    active_index: Rc<Cell<Option<usize>>>,
    row_meta: Rc<RefCell<Arc<[TableNavRowMeta]>>>,
    active_element: Rc<Cell<Option<fret_ui::GlobalElementId>>>,
    active_command: Rc<RefCell<Option<CommandId>>>,
}

#[allow(clippy::too_many_arguments)]
pub fn table_virtualized<H: UiHost, TData>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: &[ColumnDef<TData>],
    state: Model<TableState>,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: &dyn Fn(&TData, usize) -> RowKey,
    props: TableViewProps,
    on_row_activate: impl Fn(&Row<'_, TData>) -> Option<CommandId>,
    mut render_header_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &ColumnDef<TData>,
        Option<bool>,
    ) -> Vec<AnyElement>,
    mut render_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &Row<'_, TData>,
        &ColumnDef<TData>,
    ) -> Vec<AnyElement>,
    output: Option<Model<TableViewOutput>>,
) -> AnyElement {
    let profile = std::env::var_os("FRET_TABLE_PROFILE").is_some();
    let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();

    let theme = Theme::global(&*cx.app);
    let (table_bg, border, header_bg, row_hover, row_active) = resolve_table_colors(theme);
    let resize_grip = emphasize_border(border, 0.35);
    let resize_preview = emphasize_border(border, 0.75);
    let radius = theme.metric_required("metric.radius.md");

    let row_h = props
        .row_height
        .unwrap_or_else(|| resolve_row_height(theme, props.size));
    let cell_px = resolve_cell_padding_x(theme);
    let cell_py = resolve_cell_padding_y(theme);

    let scroll_x = cx.with_state(ScrollHandle::default, |h| h.clone());

    let grouping = if props.enable_column_grouping {
        state_value.grouping.as_slice()
    } else {
        &[]
    };

    let ordered_columns = order_columns(columns, &state_value.column_order);
    let ordered_columns = order_column_refs_for_grouping(
        ordered_columns.as_slice(),
        grouping,
        props.grouped_column_mode,
    );
    let visible_columns = ordered_columns
        .into_iter()
        .filter(|c| is_column_visible(&state_value.column_visibility, &c.id))
        .collect::<Vec<_>>();
    let (left_cols, center_cols, right_cols) =
        split_pinned_columns(visible_columns.as_slice(), &state_value.column_pinning);

    let sorting_key = if grouping.is_empty() {
        state_value.sorting.clone()
    } else {
        Vec::new()
    };

    let row_order = cx.with_state(FlatRowOrderCache::default, |cache| {
        let deps = FlatRowOrderDeps {
            items_revision,
            data_len: data.len(),
            sorting: sorting_key.clone(),
            column_filters: state_value.column_filters.clone(),
            global_filter: state_value.global_filter.clone(),
        };

        let started = Instant::now();
        let (order, recomputed) = cache.row_order(data, &columns, deps);
        let elapsed = started.elapsed();

        if profile && recomputed {
            tracing::info!(
                "table_virtualized: recompute row_order len={} sorting={} took {:.2}ms",
                data.len(),
                sorting_key.len(),
                elapsed.as_secs_f64() * 1000.0
            );
        }

        order.clone()
    });

    let page_display_rows: Vec<DisplayRow> = if grouping.is_empty() {
        let total_rows = row_order.len();
        let bounds = pagination_bounds(total_rows, state_value.pagination);
        if bounds.page_index != state_value.pagination.page_index {
            let _ = cx.app.models_mut().update(&state, |st| {
                st.pagination.page_index = bounds.page_index;
            });
        }
        if let Some(out) = output.clone() {
            let next = TableViewOutput {
                filtered_row_count: total_rows,
                pagination: bounds,
            };
            let _ = cx.app.models_mut().update(&out, |v| {
                if *v != next {
                    *v = next;
                }
            });
        }

        let page_rows: &[usize] = if bounds.page_count == 0 {
            &[]
        } else {
            row_order
                .get(bounds.page_start..bounds.page_end)
                .unwrap_or_default()
        };
        page_rows
            .iter()
            .map(|&data_index| DisplayRow::Leaf {
                data_index,
                row_key: row_key_at(&data[data_index], data_index),
                depth: 0,
            })
            .collect()
    } else {
        let deps = GroupedDisplayDeps {
            base: GroupedBaseDeps {
                items_revision,
                data_len: data.len(),
                columns_fingerprint: columns_fingerprint(columns),
                grouping: grouping.to_vec(),
                column_filters: state_value.column_filters.clone(),
                global_filter: state_value.global_filter.clone(),
            },
            sorting: state_value.sorting.clone(),
            expanding: state_value.expanding.clone(),
            page_index: state_value.pagination.page_index,
            page_size: state_value.pagination.page_size,
        };

        let (page_rows, view_output, clamp_to_page): (
            Vec<DisplayRow>,
            TableViewOutput,
            Option<usize>,
        ) = cx.with_state(GroupedDisplayCache::default, |cache| {
            if cache.deps.as_ref() == Some(&deps) {
                return (cache.page_rows.clone(), cache.output.clone(), None);
            }

            if cache.base_deps.as_ref() == Some(&deps.base) {
                let grouped = &cache.grouped;
                let row_index_by_key = &cache.row_index_by_key;
                let group_labels = &cache.group_labels;
                let group_aggs_text = &cache.group_aggs_text;
                let group_aggs_u64 = &cache.group_aggs_u64;

                let mut visible: Vec<DisplayRow> = Vec::new();
                let mut roots: Vec<crate::headless::table::GroupedRowIndex> =
                    grouped.root_rows().to_vec();
                sort_grouped_row_indices_in_place(
                    grouped,
                    &mut roots,
                    deps.sorting.as_slice(),
                    columns,
                    data,
                    row_index_by_key,
                    group_aggs_u64,
                );

                for root in roots {
                    push_visible(
                        grouped,
                        root,
                        row_index_by_key,
                        group_labels,
                        group_aggs_text,
                        group_aggs_u64,
                        deps.sorting.as_slice(),
                        columns,
                        data,
                        &deps.expanding,
                        &mut visible,
                    );
                }

                let total_rows = visible.len();
                let bounds = pagination_bounds(
                    total_rows,
                    PaginationState {
                        page_index: deps.page_index,
                        page_size: deps.page_size,
                    },
                );
                cache.output = TableViewOutput {
                    filtered_row_count: total_rows,
                    pagination: bounds,
                };

                let page_rows: Vec<DisplayRow> = if bounds.page_count == 0 {
                    Vec::new()
                } else {
                    visible
                        .get(bounds.page_start..bounds.page_end)
                        .unwrap_or_default()
                        .to_vec()
                };

                cache.deps = Some(deps.clone());
                cache.page_rows = page_rows.clone();
                return (
                    page_rows,
                    cache.output.clone(),
                    (bounds.page_index != deps.page_index).then_some(bounds.page_index),
                );
            }

            let mut row_index_by_key: std::collections::HashMap<RowKey, usize> =
                std::collections::HashMap::with_capacity(data.len());
            for (i, item) in data.iter().enumerate() {
                let key = row_key_at(item, i);
                row_index_by_key.entry(key).or_insert(i);
            }

            let col_by_id: std::collections::HashMap<&str, &ColumnDef<TData>> =
                columns.iter().map(|c| (c.id.as_ref(), c)).collect();

            let agg_columns: Vec<&ColumnDef<TData>> = columns
                .iter()
                .filter(|c| c.aggregation != Aggregation::None)
                .collect();

            let mut options = crate::headless::table::TableOptions::default();
            options.manual_sorting = true;
            options.manual_pagination = true;
            options.manual_expanding = true;

            let mut state_for_grouping = state_value.clone();
            state_for_grouping.sorting.clear();
            state_for_grouping.pagination = Default::default();

            let table = Table::builder(data)
                .columns(columns.to_vec())
                .state(state_for_grouping)
                .options(options)
                .get_row_key(|row, index, _parent| row_key_at(row, index))
                .build();

            let grouped = table.grouped_row_model().clone();
            fn compute_group_aggregations<TData>(
                model: &crate::headless::table::GroupedRowModel,
                data: &[TData],
                row_index_by_key: &std::collections::HashMap<RowKey, usize>,
                agg_columns: &[&ColumnDef<TData>],
            ) -> (
                std::collections::HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
                std::collections::HashMap<RowKey, Arc<[(ColumnId, Arc<str>)]>>,
            ) {
                if agg_columns.is_empty() {
                    return (Default::default(), Default::default());
                }
                let out_u64 =
                    compute_grouped_u64_aggregations(model, data, row_index_by_key, agg_columns);

                let mut out_text: std::collections::HashMap<RowKey, Arc<[(ColumnId, Arc<str>)]>> =
                    Default::default();
                for (&row_key, entries) in &out_u64 {
                    let mut text_values: Vec<(ColumnId, Arc<str>)> =
                        Vec::with_capacity(entries.len());
                    for (col_id, v) in entries.iter() {
                        text_values.push((col_id.clone(), Arc::from(v.to_string())));
                    }
                    out_text.insert(row_key, Arc::from(text_values.into_boxed_slice()));
                }

                (out_u64, out_text)
            }

            fn group_label_for_key<TData>(
                kind: &GroupedRowKind,
                data: &[TData],
                row_index_by_key: &std::collections::HashMap<RowKey, usize>,
                col_by_id: &std::collections::HashMap<&str, &ColumnDef<TData>>,
            ) -> Arc<str> {
                let GroupedRowKind::Group {
                    grouping_column,
                    grouping_value,
                    first_leaf_row_key,
                    leaf_row_count,
                } = kind
                else {
                    return Arc::from("");
                };

                let mut value: Arc<str> = Arc::from(format!("{:x}", grouping_value));
                if let Some(column) = col_by_id.get(grouping_column.as_ref()).copied()
                    && let Some(f) = column.facet_str_fn.as_ref()
                    && let Some(&i) = row_index_by_key.get(first_leaf_row_key)
                {
                    value = Arc::from(f(&data[i]));
                }

                Arc::from(format!("{value} ({leaf_row_count})"))
            }

            fn push_visible<'a, TData>(
                model: &'a crate::headless::table::GroupedRowModel,
                index: crate::headless::table::GroupedRowIndex,
                row_index_by_key: &std::collections::HashMap<RowKey, usize>,
                group_labels: &std::collections::HashMap<RowKey, Arc<str>>,
                group_aggs_text: &std::collections::HashMap<RowKey, Arc<[(ColumnId, Arc<str>)]>>,
                group_aggs_u64: &std::collections::HashMap<RowKey, Arc<[(ColumnId, u64)]>>,
                sorting: &[SortSpec],
                columns: &[ColumnDef<TData>],
                data: &[TData],
                expanded: &ExpandingState,
                out: &mut Vec<DisplayRow>,
            ) {
                let Some(row) = model.row(index) else {
                    return;
                };

                match &row.kind {
                    GroupedRowKind::Group {
                        grouping_column, ..
                    } => {
                        let expanded_here = is_row_expanded(row.key, expanded);
                        let grouping_column = grouping_column.clone();
                        let aggregations = group_aggs_text
                            .get(&row.key)
                            .cloned()
                            .unwrap_or_else(|| Arc::from([]));
                        out.push(DisplayRow::Group {
                            grouping_column,
                            row_key: row.key,
                            depth: row.depth,
                            label: group_labels
                                .get(&row.key)
                                .cloned()
                                .unwrap_or_else(|| Arc::from("")),
                            expanded: expanded_here,
                            aggregations,
                        });

                        if expanded_here {
                            let mut children: Option<Vec<crate::headless::table::GroupedRowIndex>> =
                                None;

                            if let Some(spec) = sorting.first() {
                                let mut owned = row.sub_rows.clone();
                                sort_grouped_row_indices_in_place(
                                    model,
                                    &mut owned,
                                    std::slice::from_ref(spec),
                                    columns,
                                    data,
                                    row_index_by_key,
                                    group_aggs_u64,
                                );
                                children = Some(owned);
                            }

                            let child_iter: Box<
                                dyn Iterator<Item = crate::headless::table::GroupedRowIndex>,
                            > = if let Some(children) = children {
                                Box::new(children.into_iter())
                            } else {
                                Box::new(row.sub_rows.iter().copied())
                            };

                            for child in child_iter {
                                push_visible(
                                    model,
                                    child,
                                    row_index_by_key,
                                    group_labels,
                                    group_aggs_text,
                                    group_aggs_u64,
                                    sorting,
                                    columns,
                                    data,
                                    expanded,
                                    out,
                                );
                            }
                        }
                    }
                    GroupedRowKind::Leaf { row_key } => {
                        let Some(&data_index) = row_index_by_key.get(row_key) else {
                            return;
                        };
                        out.push(DisplayRow::Leaf {
                            data_index,
                            row_key: *row_key,
                            depth: row.depth,
                        });
                    }
                }
            }

            let (group_aggs_u64, group_aggs_text) =
                compute_group_aggregations(&grouped, data, &row_index_by_key, &agg_columns);

            let mut group_labels: std::collections::HashMap<RowKey, Arc<str>> = Default::default();
            for &node in grouped.flat_rows() {
                let Some(row) = grouped.row(node) else {
                    continue;
                };
                if matches!(row.kind, GroupedRowKind::Group { .. }) {
                    group_labels.insert(
                        row.key,
                        group_label_for_key(&row.kind, data, &row_index_by_key, &col_by_id),
                    );
                }
            }

            let mut visible: Vec<DisplayRow> = Vec::new();
            let mut roots: Vec<crate::headless::table::GroupedRowIndex> =
                grouped.root_rows().to_vec();
            sort_grouped_row_indices_in_place(
                &grouped,
                &mut roots,
                deps.sorting.as_slice(),
                columns,
                data,
                &row_index_by_key,
                &group_aggs_u64,
            );

            for root in roots {
                push_visible(
                    &grouped,
                    root,
                    &row_index_by_key,
                    &group_labels,
                    &group_aggs_text,
                    &group_aggs_u64,
                    deps.sorting.as_slice(),
                    columns,
                    data,
                    &state_value.expanding,
                    &mut visible,
                );
            }

            let total_rows = visible.len();
            let bounds = pagination_bounds(total_rows, state_value.pagination);

            cache.output = TableViewOutput {
                filtered_row_count: total_rows,
                pagination: bounds,
            };

            let page_rows: Vec<DisplayRow> = if bounds.page_count == 0 {
                Vec::new()
            } else {
                visible
                    .get(bounds.page_start..bounds.page_end)
                    .unwrap_or_default()
                    .to_vec()
            };

            cache.base_deps = Some(deps.base.clone());
            cache.grouped = grouped;
            cache.row_index_by_key = row_index_by_key;
            cache.group_labels = group_labels;
            cache.group_aggs_u64 = group_aggs_u64;
            cache.group_aggs_text = group_aggs_text;
            cache.deps = Some(deps.clone());
            cache.page_rows = page_rows.clone();
            (
                page_rows,
                cache.output.clone(),
                (bounds.page_index != deps.page_index).then_some(bounds.page_index),
            )
        });

        if let Some(page_index) = clamp_to_page {
            let _ = cx.app.models_mut().update(&state, |st| {
                st.pagination.page_index = page_index;
            });
        }
        if let Some(out) = output {
            let _ = cx.app.models_mut().update(&out, |v| {
                if *v != view_output {
                    *v = view_output;
                }
            });
        }

        page_rows
    };

    let set_size = page_display_rows.len();

    let mut list_options = fret_ui::element::VirtualListOptions::new(row_h, props.overscan);
    list_options.items_revision = items_revision;
    list_options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;
    list_options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;

    let rendered_rows = Cell::new(0usize);

    let (active_index, row_meta, active_element, active_command) =
        cx.with_state(TableKeyboardNavState::default, |st| {
            (
                st.active_index.clone(),
                st.row_meta.clone(),
                st.active_element.clone(),
                st.active_command.clone(),
            )
        });

    {
        let next_meta: Arc<[TableNavRowMeta]> = page_display_rows
            .iter()
            .map(|row| match row {
                DisplayRow::Leaf { row_key, .. } => TableNavRowMeta {
                    row_key: *row_key,
                    kind: TableNavRowKind::Leaf,
                },
                DisplayRow::Group { row_key, .. } => TableNavRowMeta {
                    row_key: *row_key,
                    kind: TableNavRowKind::Group,
                },
            })
            .collect::<Vec<_>>()
            .into();
        *row_meta.borrow_mut() = next_meta;
    }

    if set_size == 0 {
        if active_index.get().is_some() {
            active_index.set(None);
        }
    } else {
        let next = Some(
            active_index
                .get()
                .unwrap_or(0)
                .min(set_size.saturating_sub(1)),
        );
        if active_index.get() != next {
            active_index.set(next);
        }
    }

    let key_handler: fret_ui::action::OnKeyDown = {
        let active_index = active_index.clone();
        let row_meta = row_meta.clone();
        let active_command = active_command.clone();
        let vertical_scroll = vertical_scroll.clone();
        let state = state.clone();
        let enable_row_selection = props.enable_row_selection;
        let single_row_selection = props.single_row_selection;

        Arc::new(move |host, action_cx, down| {
            if down.modifiers != Modifiers::default() {
                return false;
            }

            let meta = row_meta.borrow().clone();
            let len = meta.len();
            if len == 0 {
                if active_index.get().is_some() {
                    active_index.set(None);
                    host.request_redraw(action_cx.window);
                }
                return false;
            }

            let current = active_index.get().unwrap_or(0).min(len.saturating_sub(1));

            match down.key {
                KeyCode::ArrowDown
                | KeyCode::ArrowUp
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageDown
                | KeyCode::PageUp => {
                    let page = 10usize;
                    let next = match down.key {
                        KeyCode::ArrowDown => (current + 1).min(len.saturating_sub(1)),
                        KeyCode::ArrowUp => current.saturating_sub(1),
                        KeyCode::Home => 0,
                        KeyCode::End => len.saturating_sub(1),
                        KeyCode::PageDown => (current + page).min(len.saturating_sub(1)),
                        KeyCode::PageUp => current.saturating_sub(page),
                        _ => current,
                    };

                    if next != current {
                        active_index.set(Some(next));
                        *active_command.borrow_mut() = None;
                        vertical_scroll.scroll_to_item(next, ScrollStrategy::Nearest);
                        host.request_redraw(action_cx.window);
                    }
                    true
                }
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                    let Some(meta) = meta.get(current).copied() else {
                        return false;
                    };

                    if let Some(cmd) = active_command.borrow().clone() {
                        host.dispatch_command(Some(action_cx.window), cmd);
                    }

                    match meta.kind {
                        TableNavRowKind::Group => {
                            let row_key = meta.row_key;
                            let _ = host.models_mut().update(&state, move |st| {
                                match &mut st.expanding {
                                    ExpandingState::All => {
                                        st.expanding = ExpandingState::default();
                                    }
                                    ExpandingState::Keys(keys) => {
                                        if keys.contains(&row_key) {
                                            keys.remove(&row_key);
                                        } else {
                                            keys.insert(row_key);
                                        }
                                    }
                                }
                            });
                            host.request_redraw(action_cx.window);
                            true
                        }
                        TableNavRowKind::Leaf => {
                            if !enable_row_selection {
                                return false;
                            }

                            let row_key = meta.row_key;
                            let _ = host.models_mut().update(&state, move |st| {
                                let selected = st.row_selection.contains(&row_key);
                                if single_row_selection {
                                    st.row_selection.clear();
                                }
                                if selected {
                                    st.row_selection.remove(&row_key);
                                } else {
                                    st.row_selection.insert(row_key);
                                }
                            });
                            host.request_redraw(action_cx.window);
                            true
                        }
                    }
                }
                _ => false,
            }
        })
    };

    let active_descendant = active_element.get().and_then(|id| cx.node_for_element(id));

    cx.semantics_with_id(
        SemanticsProps {
            role: SemanticsRole::List,
            focusable: true,
            active_descendant,
            ..Default::default()
        },
        |cx, list_id| {
            cx.key_on_key_down_for(list_id, key_handler.clone());
            vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout.flex.grow = 1.0;
                        layout.flex.basis = Length::Px(Px(0.0));
                        layout.overflow = Overflow::Clip;
                        layout
                    },
                    background: Some(table_bg),
                    border: if props.draw_frame {
                        Edges::all(Px(1.0))
                    } else {
                        Edges::all(Px(0.0))
                    },
                    border_color: if props.draw_frame { Some(border) } else { None },
                    corner_radii: if props.draw_frame {
                        Corners::all(radius)
                    } else {
                        Corners::all(Px(0.0))
                    },
                    ..Default::default()
                },
                |cx| {
                    vec![stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().size_full())
                            .gap_y(Space::N0)
                            .justify(Justify::Start)
                            .items(Items::Stretch),
                        |cx| {
                                    let header = cx.container(
                                        ContainerProps {
                                            background: Some(header_bg),
                                            border: Edges {
                                                bottom: Px(1.0),
                                                ..Default::default()
                                            },
                                            border_color: Some(border),
                                            layout: LayoutStyle {
                                                size: fret_ui::element::SizeStyle {
                                                    height: Length::Px(row_h),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        |cx| {
                                            let mut render_header_group =
                                                |cx: &mut ElementContext<'_, H>,
                                                 cols: &[&ColumnDef<TData>],
                                                 scroll_x: Option<ScrollHandle>| {
                                                    let row = stack::hstack(
                                                        cx,
                                                        stack::HStackProps::default()
                                                            .gap_x(Space::N0)
                                                            .justify(Justify::Start)
                                                            .items(Items::Center),
                                                        |cx| {
                                                            let out: Vec<AnyElement> = cols.iter()
                                                                .map(|col| {
                                                                    let sort_state = sort_for_column(
                                                                        &state_value.sorting,
                                                                        &col.id,
                                                                    );

                                                                    let col_w = resolve_column_width(
                                                                        col,
                                                                        &state_value,
                                                                        &props,
                                                                    );

                                                                    let cell_props = ContainerProps {
                                                                        padding: Edges::all(Px(0.0)),
                                                                        border: Edges {
                                                                            right: if props.enable_column_resizing
                                                                                && col.enable_resizing
                                                                            {
                                                                                Px(0.0)
                                                                            } else {
                                                                                Px(1.0)
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                        border_color: Some(border),
                                                                        layout: LayoutStyle {
                                                                            size: fret_ui::element::SizeStyle {
                                                                                width: Length::Px(col_w),
                                                                                ..Default::default()
                                                                            },
                                                                            flex: fret_ui::element::FlexItemStyle {
                                                                                shrink: 0.0,
                                                                                ..Default::default()
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    };

                                                                    cx.container(cell_props, |cx| {
                                                                        let mut out = Vec::new();

                                                                        out.push(stack::hstack(
                                                                            cx,
                                                                            stack::HStackProps::default()
                                                                                .layout(
                                                                                    LayoutRefinement::default()
                                                                                        .size_full()
                                                                                        .relative(),
                                                                                )
                                                                                .gap_x(Space::N0)
                                                                                .justify(Justify::Start)
                                                                                .items(Items::Center),
                                                                            |cx| {
                                                                                let mut pieces = Vec::new();

                                                                                let enabled = props.enable_sorting
                                                                                    && col.sort_cmp.is_some();
                                                                                let col_id = col.id.clone();
                                                                                let state_model =
                                                                                    state.clone();

                                                                                pieces.push(cx.pressable(
                                                                                    PressableProps {
                                                                                        layout: {
                                                                                            let mut layout = LayoutStyle::default();
                                                                                            layout.size.width = Length::Fill;
                                                                                            layout.size.height = Length::Fill;
                                                                                            layout.flex.grow = 1.0;
                                                                                            layout.flex.shrink = 1.0;
                                                                                            layout.flex.basis = Length::Px(Px(0.0));
                                                                                            layout
                                                                                        },
                                                                                        enabled,
                                                                                        a11y: PressableA11y {
                                                                                            role: Some(
                                                                                                SemanticsRole::Button,
                                                                                            ),
                                                                                            ..Default::default()
                                                                                        },
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |cx, _| {
                                                                                        if enabled {
                                                                                            cx.pressable_update_model(
                                                                                                &state_model,
                                                                                                move |st| {
                                                                                                    apply_single_sort_toggle(
                                                                                                        st,
                                                                                                        &col_id,
                                                                                                    );
                                                                                                },
                                                                                            );
                                                                                        }

                                                                                        let cell =
                                                                                            render_header_cell(cx, col, sort_state);
                                                                                        vec![cx.container(
                                                                                            ContainerProps {
                                                                                                padding: Edges::symmetric(
                                                                                                    cell_px, cell_py,
                                                                                                ),
                                                                                                layout: {
                                                                                                    let mut layout =
                                                                                                        LayoutStyle::default();
                                                                                                    layout.size.width =
                                                                                                        Length::Fill;
                                                                                                    layout.size.height =
                                                                                                        Length::Fill;
                                                                                                    layout
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            |_cx| cell,
                                                                                        )]
                                                                                    },
                                                                                ));

                                                                                if props.enable_column_resizing
                                                                                    && col.enable_resizing
                                                                                {
                                                                                    let col_id = col.id.clone();
                                                                                    let state_model = state.clone();
                                                                                    let default_w = props.default_column_width;
                                                                                    let min_w = col.min_size.max(props.min_column_width.0).max(0.0);
                                                                                    let max_w = col.max_size.max(min_w);
                                                                                    let resize_mode = props.column_resize_mode;
                                                                                    let resize_direction = props.column_resize_direction;
                                                                                    let grip_color = resize_grip;

                                                                                    if props.enable_column_resizing
                                                                                        && props.column_resize_mode
                                                                                            == ColumnResizeMode::OnEnd
                                                                                        && state_value
                                                                                            .column_sizing_info
                                                                                            .is_resizing_column
                                                                                            .as_ref()
                                                                                            .is_some_and(|active| {
                                                                                                active.as_ref()
                                                                                                    == col_id.as_ref()
                                                                                            })
                                                                                    {
                                                                                        let delta = state_value
                                                                                            .column_sizing_info
                                                                                            .delta_offset
                                                                                            .unwrap_or(0.0);
                                                                                        pieces.push(cx.container(
                                                                                            ContainerProps {
                                                                                                background: Some(
                                                                                                    resize_preview,
                                                                                                ),
                                                                                                layout: LayoutStyle {
                                                                                                    size:
                                                                                                        fret_ui::element::SizeStyle {
                                                                                                            width: Length::Px(Px(2.0)),
                                                                                                            height: Length::Fill,
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                    position: fret_ui::element::PositionStyle::Absolute,
                                                                                                    inset: fret_ui::element::InsetStyle {
                                                                                                        top: Some(Px(0.0)),
                                                                                                        right: Some(Px(-delta - 1.0)),
                                                                                                        bottom: Some(Px(0.0)),
                                                                                                        left: None,
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            |_| Vec::new(),
                                                                                        ));
                                                                                    }

                                                                                    pieces.push(cx.pointer_region(
                                                                                        PointerRegionProps {
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    width: Length::Px(Px(12.0)),
                                                                                                    height: Length::Fill,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                position:
                                                                                                    fret_ui::element::PositionStyle::Absolute,
                                                                                                inset: fret_ui::element::InsetStyle {
                                                                                                    top: Some(Px(0.0)),
                                                                                                    right: Some(Px(0.0)),
                                                                                                    bottom: Some(Px(0.0)),
                                                                                                    left: None,
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            enabled: true,
                                                                                        },
                                                                                        |cx| {
                                                                                            let state_model_down = state_model.clone();
                                                                                            let state_model_move = state_model.clone();
                                                                                            let state_model_up = state_model.clone();
                                                                                            let col_id_down = col_id.clone();
                                                                                            let col_id_move = col_id.clone();
                                                                                            let col_id_up = col_id.clone();

                                                                                            cx.pointer_region_on_pointer_down(
                                                                                                std::sync::Arc::new(move |host, _acx, down| {
                                                                                                    if down.button != fret_core::MouseButton::Left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    host.capture_pointer();
                                                                                                    host.set_cursor_icon(CursorIcon::ColResize);
                                                                                                    let _ = host.models_mut().update(&state_model_down, |st| {
                                                                                                        let start = st
                                                                                                            .column_sizing
                                                                                                            .get(&col_id_down)
                                                                                                            .copied()
                                                                                                            .unwrap_or(default_w.0)
                                                                                                            .clamp(min_w, max_w);
                                                                                                        st.column_sizing.insert(col_id_down.clone(), start);
                                                                                                        begin_column_resize(
                                                                                                            &mut st.column_sizing_info,
                                                                                                            col_id_down.clone(),
                                                                                                            down.position.x.0,
                                                                                                            vec![(col_id_down.clone(), start)],
                                                                                                        );
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            cx.pointer_region_on_pointer_move(
                                                                                                std::sync::Arc::new(move |host, _acx, mv| {
                                                                                                    host.set_cursor_icon(CursorIcon::ColResize);
                                                                                                    if !mv.buttons.left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    let _ = host.models_mut().update(&state_model_move, |st| {
                                                                                                        let Some(active) = &st.column_sizing_info.is_resizing_column else { return; };
                                                                                                        if active.as_ref() != col_id_move.as_ref() { return; }
                                                                                                        drag_column_resize(
                                                                                                            resize_mode,
                                                                                                            resize_direction,
                                                                                                            &mut st.column_sizing,
                                                                                                            &mut st.column_sizing_info,
                                                                                                            mv.position.x.0,
                                                                                                        );
                                                                                                        if let Some(next) = st.column_sizing.get(&col_id_move).copied() {
                                                                                                            st.column_sizing.insert(col_id_move.clone(), next.clamp(min_w, max_w));
                                                                                                        }
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            cx.pointer_region_on_pointer_up(
                                                                                                std::sync::Arc::new(move |host, _acx, up| {
                                                                                                    if up.button != fret_core::MouseButton::Left {
                                                                                                        return false;
                                                                                                    }
                                                                                                    host.release_pointer_capture();
                                                                                                    let _ = host.models_mut().update(&state_model_up, |st| {
                                                                                                        if !st
                                                                                                            .column_sizing_info
                                                                                                            .is_resizing_column
                                                                                                            .as_ref()
                                                                                                            .is_some_and(|a| a.as_ref() == col_id_up.as_ref())
                                                                                                        {
                                                                                                            return;
                                                                                                        }
                                                                                                        end_column_resize(
                                                                                                            resize_mode,
                                                                                                            resize_direction,
                                                                                                            &mut st.column_sizing,
                                                                                                            &mut st.column_sizing_info,
                                                                                                            Some(up.position.x.0),
                                                                                                        );
                                                                                                        if let Some(next) = st.column_sizing.get(&col_id_up).copied() {
                                                                                                            st.column_sizing.insert(col_id_up.clone(), next.clamp(min_w, max_w));
                                                                                                        }
                                                                                                    });
                                                                                                    true
                                                                                                }),
                                                                                            );
                                                                                            vec![stack::hstack(
                                                                                                cx,
                                                                                                stack::HStackProps::default()
                                                                                                    .gap_x(Space::N0)
                                                                                                    .justify(Justify::End)
                                                                                                    .items(Items::Stretch),
                                                                                                |cx| {
                                                                                                    vec![cx.container(
                                                                                                        ContainerProps {
                                                                                                            background: Some(grip_color),
                                                                                                            layout: LayoutStyle {
                                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                                    width: Length::Px(Px(1.0)),
                                                                                                                    height: Length::Fill,
                                                                                                                    ..Default::default()
                                                                                                                },
                                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                                    shrink: 0.0,
                                                                                                                    ..Default::default()
                                                                                                                },
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                        |_| Vec::new(),
                                                                                                    )]
                                                                                                },
                                                                                            )]
                                                                                        },
                                                                                    ));
                                                                                }

                                                                                pieces
                                                                            },
                                                                        ));

                                                                        out
                                                                    })
                                                                })
                                                                .collect();

                                                            out
                                                        },
                                                    );

                                                            if let Some(scroll_x) = scroll_x {
                                                                cx.scroll(
                                                                    ScrollProps {
                                                                        axis: ScrollAxis::X,
                                                                        scroll_handle: Some(scroll_x),
                                                                        layout: LayoutStyle {
                                                                            size: fret_ui::element::SizeStyle {
                                                                                width: Length::Fill,
                                                                                height: Length::Fill,
                                                                                ..Default::default()
                                                                            },
                                                                            flex: fret_ui::element::FlexItemStyle {
                                                                                grow: 1.0,
                                                                                shrink: 1.0,
                                                                                basis: Length::Px(Px(0.0)),
                                                                                ..Default::default()
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    |_| vec![row],
                                                                )
                                                            } else {
                                                                row
                                                            }
                                                };

                                            vec![stack::hstack(
                                                cx,
                                                stack::HStackProps::default()
                                                    .gap_x(Space::N0)
                                                    .justify(Justify::Start)
                                                    .items(Items::Stretch),
                                                |cx| {
                                                    let has_left = !left_cols.is_empty();
                                                    let has_center = !center_cols.is_empty();
                                                    let has_right = !right_cols.is_empty();

                                                    let divider_after_left = props.optimize_grid_lines
                                                        && has_left
                                                        && (has_center || has_right);
                                                    let divider_after_center =
                                                        props.optimize_grid_lines && has_center && has_right;

                                                    let left = render_header_group(cx, &left_cols, None);
                                                    let left = if divider_after_left {
                                                        cx.container(
                                                            ContainerProps {
                                                                border: Edges {
                                                                    right: Px(1.0),
                                                                    ..Default::default()
                                                                },
                                                                border_color: Some(border),
                                                                layout: LayoutStyle {
                                                                    size: fret_ui::element::SizeStyle {
                                                                        height: Length::Fill,
                                                                        ..Default::default()
                                                                    },
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            move |_| vec![left],
                                                        )
                                                    } else {
                                                        left
                                                    };

                                                    let center = render_header_group(
                                                        cx,
                                                        &center_cols,
                                                        Some(scroll_x.clone()),
                                                    );
                                                    let center = if divider_after_center {
                                                        cx.container(
                                                            ContainerProps {
                                                                border: Edges {
                                                                    right: Px(1.0),
                                                                    ..Default::default()
                                                                },
                                                                border_color: Some(border),
                                                                layout: LayoutStyle {
                                                                    size: fret_ui::element::SizeStyle {
                                                                        height: Length::Fill,
                                                                        ..Default::default()
                                                                    },
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            move |_| vec![center],
                                                        )
                                                    } else {
                                                        center
                                                    };

                                                    let right = render_header_group(cx, &right_cols, None);
                                                    vec![left, center, right]
                                                },
                                            )]
                                        },
                                    );

                                    let body = cx.virtual_list_keyed_with_layout(
                                        {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.size.height = Length::Fill;
                                            layout.flex.grow = 1.0;
                                            layout.flex.basis = Length::Px(Px(0.0));
                                            layout
                                        },
                                        set_size,
                                        list_options,
                                        vertical_scroll,
                                        |i| page_display_rows[i].row_key().0,
                                        |cx, i| {
                                            rendered_rows.set(rendered_rows.get().saturating_add(1));
                                            let (data_index, row_key, depth) =
                                                match &page_display_rows[i] {
                                                    DisplayRow::Leaf {
                                                        data_index,
                                                        row_key,
                                                        depth,
                                                    } => (
                                                        *data_index,
                                                        *row_key,
                                                        u16::try_from(*depth).unwrap_or(u16::MAX),
                                                    ),
                                                    DisplayRow::Group {
                                                        grouping_column,
                                                        row_key,
                                                        depth,
                                                        label,
                                                        expanded,
                                                        aggregations,
                                                    } => {
                                                        let row_key = *row_key;
                                                        let depth = *depth;
                                                        let expanded = *expanded;
                                                        let label = label.clone();
                                                        let grouping_column = grouping_column.clone();
                                                        let aggregations = aggregations.clone();

                                                        let label_target: ColumnId = if left_cols
                                                            .iter()
                                                            .chain(center_cols.iter())
                                                            .chain(right_cols.iter())
                                                            .any(|c| c.id.as_ref() == grouping_column.as_ref())
                                                        {
                                                            grouping_column.clone()
                                                        } else {
                                                            left_cols
                                                                .first()
                                                                .or_else(|| center_cols.first())
                                                                .or_else(|| right_cols.first())
                                                                .map(|c| c.id.clone())
                                                                .unwrap_or_else(|| grouping_column.clone())
                                                        };

                                                        let enabled = true;
                                                        let active_index = active_index.clone();
                                                        let active_element = active_element.clone();
                                                        let active_command = active_command.clone();
                                                        let key_handler = key_handler.clone();
                                                        let focus_target = list_id;

                                                        return cx.pressable(
                                                            PressableProps {
                                                                enabled,
                                                                focusable: false,
                                                                a11y: PressableA11y {
                                                                    role: Some(
                                                                        SemanticsRole::ListItem,
                                                                    ),
                                                                    expanded: Some(expanded),
                                                                    ..Default::default()
                                                                }
                                                                .with_collection_position(i, set_size),
                                                                ..Default::default()
                                                            },
                                                            |cx, st| {
                                                                cx.key_on_key_down_for(
                                                                    cx.root_id(),
                                                                    key_handler.clone(),
                                                                );

                                                                let active_index_for_pointer =
                                                                    active_index.clone();
                                                                let active_command_for_pointer =
                                                                    active_command.clone();
                                                                cx.pressable_on_pointer_down(
                                                                    Arc::new(move |host, action_cx, _down| {
                                                                        host.request_focus(focus_target);
                                                                        active_index_for_pointer.set(Some(i));
                                                                        *active_command_for_pointer.borrow_mut() = None;
                                                                        host.request_redraw(action_cx.window);
                                                                        PressablePointerDownResult::Continue
                                                                    }),
                                                                );

                                                                if active_index.get() == Some(i) {
                                                                    active_element.set(Some(cx.root_id()));
                                                                    *active_command.borrow_mut() = None;
                                                                }
                                                                let state_model = state.clone();
                                                                cx.pressable_update_model(
                                                                    &state_model,
                                                                    move |st| match &mut st.expanding
                                                                    {
                                                                        ExpandingState::All => {
                                                                            st.expanding =
                                                                                ExpandingState::default();
                                                                        }
                                                                        ExpandingState::Keys(keys) => {
                                                                            if keys.contains(&row_key)
                                                                            {
                                                                                keys.remove(&row_key);
                                                                            } else {
                                                                                keys.insert(row_key);
                                                                            }
                                                                        }
                                                                    },
                                                                );

                                                                let is_active = active_index.get() == Some(i);
                                                                let bg = if st.pressed {
                                                                    Some(row_active)
                                                                } else if is_active {
                                                                    Some(row_hover)
                                                                } else if st.hovered {
                                                                    Some(row_hover)
                                                                } else {
                                                                    None
                                                                };

                                                                let indent_step = 12.0_f32;
                                                                let indent_px =
                                                                    Px((depth as f32) * indent_step);
                                                                let glyph: Arc<str> = if expanded {
                                                                    Arc::from("v")
                                                                } else {
                                                                    Arc::from(">")
                                                                };
                                                                let text: Arc<str> = Arc::from(
                                                                    format!("{glyph} {label}"),
                                                                );

                                                                vec![cx.container(
                                                                    ContainerProps {
                                                                        background: bg,
                                                                        layout: LayoutStyle {
                                                                            size:
                                                                                fret_ui::element::SizeStyle {
                                                                                    height: Length::Px(row_h),
                                                                                    ..Default::default()
                                                                                },
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    |cx| {
                                                                        let render_group =
                                                                            |cx: &mut ElementContext<
                                                                                '_,
                                                                                H,
                                                                            >,
                                                                             cols: &[&ColumnDef<
                                                                                TData,
                                                                            >],
                                                                             scroll_x: Option<
                                                                                ScrollHandle,
                                                                            >| {
                                                                                let row = if props
                                                                                    .optimize_paint_order
                                                                                {
                                                                                    cx.container(
                                                                                        ContainerProps {
                                                                                            layout: LayoutStyle {
                                                                                                size:
                                                                                                    fret_ui::element::SizeStyle {
                                                                                                        height: Length::Fill,
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        |cx| {
                                                                                            let col_widths: Vec<Px> = cols
                                                                                                .iter()
                                                                                                .map(|col| {
                                                                                                    resolve_column_width(
                                                                                                        col,
                                                                                                        &state_value,
                                                                                                        &props,
                                                                                                    )
                                                                                                })
                                                                                                .collect();
                                                                                            let background_row =
                                                                                                stack::hstack(
                                                                                                    cx,
                                                                                                    stack::HStackProps::default()
                                                                                                        .gap_x(Space::N0)
                                                                                                        .justify(Justify::Start)
                                                                                                        .items(Items::Stretch),
                                                                                                    |cx| {
                                                                                                        cols.iter()
                                                                                                            .zip(col_widths.iter().copied())
                                                                                                            .map(|(_col, col_w)| {
                                                                                                                cx.container(
                                                                                                                    ContainerProps {
                                                                                                                        border: if props.optimize_grid_lines {
                                                                                                                            Edges::default()
                                                                                                                        } else {
                                                                                                                            Edges {
                                                                                                                                right: Px(1.0),
                                                                                                                                ..Default::default()
                                                                                                                            }
                                                                                                                        },
                                                                                                                        border_color: if props.optimize_grid_lines {
                                                                                                                            None
                                                                                                                        } else {
                                                                                                                            Some(border)
                                                                                                                        },
                                                                                                                        layout: LayoutStyle {
                                                                                                                            size: fret_ui::element::SizeStyle {
                                                                                                                                width: Length::Px(col_w),
                                                                                                                                height: Length::Fill,
                                                                                                                                ..Default::default()
                                                                                                                            },
                                                                                                                            flex: fret_ui::element::FlexItemStyle {
                                                                                                                                shrink: 0.0,
                                                                                                                                ..Default::default()
                                                                                                                            },
                                                                                                                            ..Default::default()
                                                                                                                        },
                                                                                                                        ..Default::default()
                                                                                                                    },
                                                                                                                    |_| Vec::new(),
                                                                                                                )
                                                                                                            })
                                                                                                            .collect()
                                                                                                    },
                                                                                                );

                                                                                            let content_overlay = cx.container(
                                                                                                ContainerProps {
                                                                                                    layout: LayoutStyle {
                                                                                                        size: fret_ui::element::SizeStyle {
                                                                                                            width: Length::Fill,
                                                                                                            height: Length::Fill,
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                        position:
                                                                                                            fret_ui::element::PositionStyle::Absolute,
                                                                                                        inset: fret_ui::element::InsetStyle {
                                                                                                            top: Some(Px(0.0)),
                                                                                                            right: Some(Px(0.0)),
                                                                                                            bottom: Some(Px(0.0)),
                                                                                                            left: Some(Px(0.0)),
                                                                                                        },
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                |cx| {
                                                                                                    vec![stack::hstack(
                                                                                                        cx,
                                                                                                        stack::HStackProps::default()
                                                                                                            .gap_x(Space::N0)
                                                                                                            .justify(Justify::Start)
                                                                                                            .items(Items::Center),
                                                                                                        |cx| {
                                                                                                            cols.iter()
                                                                                                                .zip(col_widths.iter().copied())
                                                                                                                .map(|(col, col_w)| {

                                                                                                                    let is_label_target = col
                                                                                                                        .id
                                                                                                                        .as_ref()
                                                                                                                        == label_target.as_ref();
                                                                                                                    let is_placeholder =
                                                                                                                        !is_label_target
                                                                                                                            && grouping.iter().any(|id| {
                                                                                                                                id.as_ref() == col.id.as_ref()
                                                                                                                            });

                                                                                                                    let padding = if is_label_target {
                                                                                                                        Edges {
                                                                                                                            left: Px(
                                                                                                                                cell_px.0
                                                                                                                                    + indent_px.0,
                                                                                                                            ),
                                                                                                                            right: cell_px,
                                                                                                                            top: cell_py,
                                                                                                                            bottom: cell_py,
                                                                                                                        }
                                                                                                                    } else {
                                                                                                                        Edges::symmetric(cell_px, cell_py)
                                                                                                                    };

                                                                                                                    cx.container(
                                                                                                                        ContainerProps {
                                                                                                                            padding,
                                                                                                                            layout: LayoutStyle {
                                                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                                                    width: Length::Px(col_w),
                                                                                                                                    height: Length::Fill,
                                                                                                                                    ..Default::default()
                                                                                                                                },
                                                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                                                    shrink: 0.0,
                                                                                                                                    ..Default::default()
                                                                                                                                },
                                                                                                                                ..Default::default()
                                                                                                                            },
                                                                                                                            ..Default::default()
                                                                                                                        },
                                                                                                                        |cx| {
                                                                                                                            if is_placeholder {
                                                                                                                                return Vec::new();
                                                                                                                            }
                                                                                                                            if is_label_target {
                                                                                                                                vec![cx.text(text.clone())]
                                                                                                                            } else {
                                                                                                                                let v = aggregations
                                                                                                                                    .iter()
                                                                                                                                    .find(|entry| {
                                                                                                                                        entry.0.as_ref()
                                                                                                                                            == col
                                                                                                                                                .id
                                                                                                                                                .as_ref()
                                                                                                                                    })
                                                                                                                                    .map(|entry| entry.1.clone());
                                                                                                                                v.map(|v| vec![cx.text(v)])
                                                                                                                                    .unwrap_or_default()
                                                                                                                            }
                                                                                                                        },
                                                                                                                    )
                                                                                                                })
                                                                                                                .collect()
                                                                                                        },
                                                                                                    )]
                                                                                                },
                                                                                            );

                                                                                            vec![background_row, content_overlay]
                                                                                        },
                                                                                    )
                                                                                } else {
                                                                                    stack::hstack(
                                                                                        cx,
                                                                                        stack::HStackProps::default()
                                                                                            .gap_x(Space::N0)
                                                                                            .justify(Justify::Start)
                                                                                            .items(Items::Center),
                                                                                        |cx| {
                                                                                            cols.iter()
                                                                                                .map(|col| {
                                                                                                    let col_w = resolve_column_width(
                                                                                                        col,
                                                                                                        &state_value,
                                                                                                        &props,
                                                                                                    );

                                                                                                    let is_label_target =
                                                                                                        col.id.as_ref()
                                                                                                            == label_target.as_ref();
                                                                                                    let is_placeholder = !is_label_target
                                                                                                        && grouping.iter().any(|id| {
                                                                                                            id.as_ref() == col.id.as_ref()
                                                                                                        });

                                                                                                    let padding = if is_label_target {
                                                                                                        Edges {
                                                                                                            left: Px(
                                                                                                                cell_px.0
                                                                                                                    + indent_px.0,
                                                                                                            ),
                                                                                                            right: cell_px,
                                                                                                            top: cell_py,
                                                                                                            bottom: cell_py,
                                                                                                        }
                                                                                                    } else {
                                                                                                        Edges::symmetric(
                                                                                                            cell_px,
                                                                                                            cell_py,
                                                                                                        )
                                                                                                    };

                                                                                                    cx.container(
                                                                                                        ContainerProps {
                                                                                                            padding,
                                                                                                            border: if props.optimize_grid_lines {
                                                                                                                Edges::default()
                                                                                                            } else {
                                                                                                                Edges {
                                                                                                                    right: Px(1.0),
                                                                                                                    ..Default::default()
                                                                                                                }
                                                                                                            },
                                                                                                            border_color: if props.optimize_grid_lines {
                                                                                                                None
                                                                                                            } else {
                                                                                                                Some(border)
                                                                                                            },
                                                                                                            layout: LayoutStyle {
                                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                                    width: Length::Px(col_w),
                                                                                                                    ..Default::default()
                                                                                                                },
                                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                                    shrink: 0.0,
                                                                                                                    ..Default::default()
                                                                                                                },
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                        |cx| {
                                                                                                            if is_placeholder {
                                                                                                                return Vec::new();
                                                                                                            }
                                                                                                            if is_label_target {
                                                                                                                vec![cx.text(text.clone())]
                                                                                                            } else {
                                                                                                                let v = aggregations
                                                                                                                    .iter()
                                                                                                                    .find(|entry| {
                                                                                                                        entry.0.as_ref()
                                                                                                                            == col
                                                                                                                                .id
                                                                                                                                .as_ref()
                                                                                                                    })
                                                                                                                    .map(|entry| entry.1.clone());
                                                                                                                v.map(|v| vec![cx.text(v)])
                                                                                                                    .unwrap_or_default()
                                                                                                            }
                                                                                                        },
                                                                                                    )
                                                                                                })
                                                                                                .collect()
                                                                                        },
                                                                                    )
                                                                                };

                                                                                if let Some(scroll_x) =
                                                                                    scroll_x
                                                                                {
                                                                                    cx.scroll(
                                                                                        ScrollProps {
                                                                                            axis: ScrollAxis::X,
                                                                                            scroll_handle: Some(scroll_x),
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    width: Length::Fill,
                                                                                                    height: Length::Fill,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                flex: fret_ui::element::FlexItemStyle {
                                                                                                    grow: 1.0,
                                                                                                    shrink: 1.0,
                                                                                                    basis: Length::Px(Px(0.0)),
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        |_| vec![row],
                                                                                    )
                                                                                } else {
                                                                                    row
                                                                                }
                                                                            };

                                                                        vec![stack::hstack(
                                                                            cx,
                                                                            stack::HStackProps::default()
                                                                                .gap_x(Space::N0)
                                                                                .justify(Justify::Start)
                                                                                .items(Items::Stretch),
                                                                            |cx| {
                                                                                let has_left =
                                                                                    !left_cols.is_empty();
                                                                                let has_center =
                                                                                    !center_cols.is_empty();
                                                                                let has_right =
                                                                                    !right_cols.is_empty();

                                                                                let divider_after_left = props
                                                                                    .optimize_grid_lines
                                                                                    && has_left
                                                                                    && (has_center || has_right);
                                                                                let divider_after_center = props
                                                                                    .optimize_grid_lines
                                                                                    && has_center
                                                                                    && has_right;

                                                                                let left = render_group(
                                                                                    cx,
                                                                                    &left_cols,
                                                                                    None,
                                                                                );
                                                                                let left = if divider_after_left {
                                                                                    cx.container(
                                                                                        ContainerProps {
                                                                                            border: Edges {
                                                                                                right: Px(1.0),
                                                                                                ..Default::default()
                                                                                            },
                                                                                            border_color: Some(border),
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    height: Length::Fill,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        move |_| vec![left],
                                                                                    )
                                                                                } else {
                                                                                    left
                                                                                };

                                                                                let center = render_group(
                                                                                    cx,
                                                                                    &center_cols,
                                                                                    Some(scroll_x.clone()),
                                                                                );
                                                                                let center = if divider_after_center {
                                                                                    cx.container(
                                                                                        ContainerProps {
                                                                                            border: Edges {
                                                                                                right: Px(1.0),
                                                                                                ..Default::default()
                                                                                            },
                                                                                            border_color: Some(border),
                                                                                            layout: LayoutStyle {
                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                    height: Length::Fill,
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        move |_| vec![center],
                                                                                    )
                                                                                } else {
                                                                                    center
                                                                                };

                                                                                let right = render_group(
                                                                                    cx,
                                                                                    &right_cols,
                                                                                    None,
                                                                                );

                                                                                vec![left, center, right]
                                                                            },
                                                                        )]
                                                                    },
                                                                )]
                                                            },
                                                        );
                                                    }
                                                };

                                            let data_row = Row {
                                                key: row_key,
                                                original: &data[data_index],
                                                index: data_index,
                                                depth,
                                                parent: None,
                                                parent_key: None,
                                                sub_rows: Vec::new(),
                                            };

                                            let cmd = on_row_activate(&data_row);
                                            let enabled = cmd.is_some() || props.enable_row_selection;
                                            let is_selected =
                                                is_row_selected(data_row.key, &state_value.row_selection);

                                            let active_index = active_index.clone();
                                            let active_element = active_element.clone();
                                            let active_command = active_command.clone();
                                            let key_handler = key_handler.clone();
                                            let focus_target = list_id;

                                            cx.pressable(
                                                PressableProps {
                                                    enabled,
                                                    focusable: false,
                                                    a11y: PressableA11y {
                                                        role: Some(SemanticsRole::ListItem),
                                                        selected: is_selected,
                                                        ..Default::default()
                                                    }
                                                    .with_collection_position(i, set_size),
                                                    ..Default::default()
                                                },
                                                |cx, st| {
                                                    cx.key_on_key_down_for(
                                                        cx.root_id(),
                                                        key_handler.clone(),
                                                    );

                                                    let active_index_for_pointer = active_index.clone();
                                                    cx.pressable_on_pointer_down(Arc::new(
                                                        move |host, action_cx, _down| {
                                                            host.request_focus(focus_target);
                                                            active_index_for_pointer.set(Some(i));
                                                            host.request_redraw(action_cx.window);
                                                            PressablePointerDownResult::Continue
                                                        },
                                                    ));

                                                    if active_index.get() == Some(i) {
                                                        active_element.set(Some(cx.root_id()));
                                                        *active_command.borrow_mut() = cmd.clone();
                                                    }
                                                    cx.pressable_dispatch_command_opt(cmd.clone());
                                                    if props.enable_row_selection {
                                                        let state_model = state.clone();
                                                        let row_key = data_row.key;
                                                        let single = props.single_row_selection;
                                                        cx.pressable_update_model(&state_model, move |st| {
                                                            let selected = st.row_selection.contains(&row_key);
                                                            if single {
                                                                st.row_selection.clear();
                                                            }
                                                            if selected {
                                                                st.row_selection.remove(&row_key);
                                                            } else {
                                                                st.row_selection.insert(row_key);
                                                            }
                                                        });
                                                    }

                                                    let is_active = active_index.get() == Some(i);
                                                    let bg = if is_selected || (enabled && st.pressed) {
                                                        Some(row_active)
                                                    } else if is_active {
                                                        Some(row_hover)
                                                    } else if enabled && st.hovered {
                                                        Some(row_hover)
                                                    } else {
                                                        None
                                                    };

                                                    vec![cx.container(
                                                        ContainerProps {
                                                            background: bg,
                                                            layout: LayoutStyle {
                                                                size: fret_ui::element::SizeStyle {
                                                                    height: Length::Px(row_h),
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            let mut render_row_group =
                                                                |cx: &mut ElementContext<'_, H>,
                                                                 cols: &[&ColumnDef<TData>],
                                                                 scroll_x: Option<ScrollHandle>| {
                                                                    let row = if props.optimize_paint_order {
                                                                        cx.container(
                                                                            ContainerProps {
                                                                                layout: LayoutStyle {
                                                                                    size: fret_ui::element::SizeStyle {
                                                                                        height: Length::Fill,
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                ..Default::default()
                                                                            },
                                                                            |cx| {
                                                                                let col_widths: Vec<Px> = cols
                                                                                    .iter()
                                                                                    .map(|col| {
                                                                                        resolve_column_width(
                                                                                            col,
                                                                                            &state_value,
                                                                                            &props,
                                                                                        )
                                                                                    })
                                                                                    .collect();
                                                                                let background_row = stack::hstack(
                                                                                    cx,
                                                                                    stack::HStackProps::default()
                                                                                        .gap_x(Space::N0)
                                                                                        .justify(Justify::Start)
                                                                                        .items(Items::Stretch),
                                                                                    |cx| {
                                                                                        cols.iter()
                                                                                            .zip(col_widths.iter().copied())
                                                                                            .map(|(_col, col_w)| {
                                                                                                cx.container(
                                                                                                    ContainerProps {
                                                                                                        border: if props.optimize_grid_lines {
                                                                                                            Edges::default()
                                                                                                        } else {
                                                                                                            Edges {
                                                                                                                right: Px(1.0),
                                                                                                                ..Default::default()
                                                                                                            }
                                                                                                        },
                                                                                                        border_color: if props.optimize_grid_lines {
                                                                                                            None
                                                                                                        } else {
                                                                                                            Some(border)
                                                                                                        },
                                                                                                        layout: LayoutStyle {
                                                                                                            size: fret_ui::element::SizeStyle {
                                                                                                                width: Length::Px(col_w),
                                                                                                                height: Length::Fill,
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            flex: fret_ui::element::FlexItemStyle {
                                                                                                                shrink: 0.0,
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    |_| Vec::new(),
                                                                                                )
                                                                                            })
                                                                                            .collect()
                                                                                    },
                                                                                );

                                                                                let content_overlay = cx.container(
                                                                                    ContainerProps {
                                                                                        layout: LayoutStyle {
                                                                                            size: fret_ui::element::SizeStyle {
                                                                                                width: Length::Fill,
                                                                                                height: Length::Fill,
                                                                                                ..Default::default()
                                                                                            },
                                                                                            position:
                                                                                                fret_ui::element::PositionStyle::Absolute,
                                                                                            inset: fret_ui::element::InsetStyle {
                                                                                                top: Some(Px(0.0)),
                                                                                                right: Some(Px(0.0)),
                                                                                                bottom: Some(Px(0.0)),
                                                                                                left: Some(Px(0.0)),
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |cx| {
                                                                                        vec![stack::hstack(
                                                                                            cx,
                                                                                            stack::HStackProps::default()
                                                                                                .gap_x(Space::N0)
                                                                                                .justify(Justify::Start)
                                                                                                .items(Items::Center),
                                                                                            |cx| {
                                                                                                cols.iter()
                                                                                                    .zip(col_widths.iter().copied())
                                                                                                    .map(|(col, col_w)| {
                                                                                                        cx.container(
                                                                                                            ContainerProps {
                                                                                                                padding: Edges::symmetric(
                                                                                                                    cell_px,
                                                                                                                    cell_py,
                                                                                                                ),
                                                                                                                layout: LayoutStyle {
                                                                                                                    size: fret_ui::element::SizeStyle {
                                                                                                                        width: Length::Px(col_w),
                                                                                                                        height: Length::Fill,
                                                                                                                        ..Default::default()
                                                                                                                    },
                                                                                                                    flex: fret_ui::element::FlexItemStyle {
                                                                                                                        shrink: 0.0,
                                                                                                                        ..Default::default()
                                                                                                                    },
                                                                                                                    ..Default::default()
                                                                                                                },
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            |cx| {
                                                                                                                render_cell(cx, &data_row, col)
                                                                                                            },
                                                                                                        )
                                                                                                    })
                                                                                                    .collect()
                                                                                            },
                                                                                        )]
                                                                                    },
                                                                                );

                                                                                vec![background_row, content_overlay]
                                                                            },
                                                                        )
                                                                    } else {
                                                                        stack::hstack(
                                                                            cx,
                                                                            stack::HStackProps::default()
                                                                                .gap_x(Space::N0)
                                                                                .justify(Justify::Start)
                                                                                .items(Items::Center),
                                                                                |cx| {
                                                                                    cols.iter()
                                                                                        .map(|col| {
                                                                                            let col_w = resolve_column_width(
                                                                                                col,
                                                                                                &state_value,
                                                                                                &props,
                                                                                            );
                                                                                            cx.container(
                                                                                                ContainerProps {
                                                                                                    padding: Edges::symmetric(
                                                                                                        cell_px, cell_py,
                                                                                                    ),
                                                                                                    border: if props.optimize_grid_lines {
                                                                                                        Edges::default()
                                                                                                    } else {
                                                                                                        Edges {
                                                                                                            right: Px(1.0),
                                                                                                            ..Default::default()
                                                                                                        }
                                                                                                    },
                                                                                                    border_color: if props.optimize_grid_lines {
                                                                                                        None
                                                                                                    } else {
                                                                                                        Some(border)
                                                                                                    },
                                                                                                    layout: LayoutStyle {
                                                                                                        size: fret_ui::element::SizeStyle {
                                                                                                            width: Length::Px(col_w),
                                                                                                            ..Default::default()
                                                                                                        },
                                                                                                    flex: fret_ui::element::FlexItemStyle {
                                                                                                        shrink: 0.0,
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            |cx| {
                                                                                                render_cell(cx, &data_row, col)
                                                                                            },
                                                                                        )
                                                                                    })
                                                                                    .collect()
                                                                            },
                                                                        )
                                                                    };

                                                                    if let Some(scroll_x) = scroll_x {
                                                                        cx.scroll(
                                                                            ScrollProps {
                                                                                axis: ScrollAxis::X,
                                                                                scroll_handle: Some(scroll_x),
                                                                                layout: LayoutStyle {
                                                                                    size: fret_ui::element::SizeStyle {
                                                                                        width: Length::Fill,
                                                                                        height: Length::Fill,
                                                                                        ..Default::default()
                                                                                    },
                                                                                    flex: fret_ui::element::FlexItemStyle {
                                                                                        grow: 1.0,
                                                                                        shrink: 1.0,
                                                                                        basis: Length::Px(Px(0.0)),
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                ..Default::default()
                                                                            },
                                                                            |_| vec![row],
                                                                        )
                                                                    } else {
                                                                        row
                                                                    }
                                                                };

                                                            vec![stack::hstack(
                                                                cx,
                                                                stack::HStackProps::default()
                                                                    .gap_x(Space::N0)
                                                                    .justify(Justify::Start)
                                                                    .items(Items::Stretch),
                                                                |cx| {
                                                                    let has_left = !left_cols.is_empty();
                                                                    let has_center = !center_cols.is_empty();
                                                                    let has_right = !right_cols.is_empty();

                                                                    let divider_after_left = props.optimize_grid_lines
                                                                        && has_left
                                                                        && (has_center || has_right);
                                                                    let divider_after_center =
                                                                        props.optimize_grid_lines && has_center && has_right;

                                                                    let left =
                                                                        render_row_group(cx, &left_cols, None);
                                                                    let left = if divider_after_left {
                                                                        cx.container(
                                                                            ContainerProps {
                                                                                border: Edges {
                                                                                    right: Px(1.0),
                                                                                    ..Default::default()
                                                                                },
                                                                                border_color: Some(border),
                                                                                layout: LayoutStyle {
                                                                                    size: fret_ui::element::SizeStyle {
                                                                                        height: Length::Fill,
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                ..Default::default()
                                                                            },
                                                                            move |_| vec![left],
                                                                        )
                                                                    } else {
                                                                        left
                                                                    };

                                                                    let center = render_row_group(
                                                                        cx,
                                                                        &center_cols,
                                                                        Some(scroll_x.clone()),
                                                                    );
                                                                    let center = if divider_after_center {
                                                                        cx.container(
                                                                            ContainerProps {
                                                                                border: Edges {
                                                                                    right: Px(1.0),
                                                                                    ..Default::default()
                                                                                },
                                                                                border_color: Some(border),
                                                                                layout: LayoutStyle {
                                                                                    size: fret_ui::element::SizeStyle {
                                                                                        height: Length::Fill,
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                ..Default::default()
                                                                            },
                                                                            move |_| vec![center],
                                                                        )
                                                                    } else {
                                                                        center
                                                                    };

                                                                    let right =
                                                                        render_row_group(cx, &right_cols, None);
                                                                    vec![left, center, right]
                                                                },
                                                            )]
                                                        },
                                                    )]
                                                },
                                            )
                                        },
                                    );

                                    if profile {
                                        tracing::info!(
                                            "table_virtualized: list len={} page_rows={} rendered_rows={} row_h={:.1}px",
                                            data.len(),
                                            set_size,
                                            rendered_rows.get(),
                                            row_h.0
                                        );
                                    }

                                    vec![header, body]
                        },
                    )]
                },
            )]
        },
    )
}
