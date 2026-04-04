use fret_core::{Color, Corners, CursorIcon, Edges, KeyCode, Px, SemanticsRole};
use fret_runtime::{CommandId, Effect, Model, ModelStore, TimerToken};
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt};
use fret_ui::element::{
    AnyElement, ContainerProps, HoverRegionProps, LayoutStyle, Length, Overflow,
    PointerRegionProps, PressableA11y, PressableProps, RingPlacement, RingStyle, ScrollAxis,
    ScrollProps, SemanticsDecoration, SemanticsProps, SpacerProps, VirtualListOptions,
};
use fret_ui::scroll::{ScrollHandle, VirtualListScrollHandle};
use fret_ui::{
    ElementContext, ElementContextAccess, GlobalElementId, Theme, UiHost, scroll::ScrollStrategy,
};

use fret_core::time::Instant;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

type TypeaheadLabelAt<TData> = dyn Fn(&TData, usize) -> Arc<str> + Send + Sync;
type CopyTextAtFn = dyn Fn(&ModelStore, usize) -> Option<String> + Send + Sync;
type RowKeyAt<TData> = dyn Fn(&TData, usize) -> RowKey;
type HeaderLabelAt<TData> = dyn Fn(&ColumnDef<TData>) -> Arc<str>;
type HeaderAccessoryAt<H, TData> =
    dyn for<'a> Fn(&mut dyn ElementContextAccess<'a, H>, &ColumnDef<TData>) -> AnyElement;
type CellAt<H, TData> =
    dyn for<'a> Fn(&mut dyn ElementContextAccess<'a, H>, &ColumnDef<TData>, &TData) -> AnyElement;
type GroupAggsU64 = std::collections::HashMap<RowKey, Arc<[(ColumnId, u64)]>>;
type GroupAggsAny = std::collections::HashMap<RowKey, Arc<[(ColumnId, TanStackValue)]>>;
type GroupAggsText = std::collections::HashMap<RowKey, Arc<[(ColumnId, Arc<str>)]>>;
type SorterFn<TData> = dyn Fn(&TData, &TData) -> std::cmp::Ordering;
type SorterSpec<TData> = (SortSpec, Arc<SorterFn<TData>>);

/// Narrow interop bridge for table surfaces that still store view state in `Model<TableState>`.
///
/// This stays intentionally table-specific so `LocalState<TableState>` can participate in the
/// public table/data-table authoring lane without widening into a crate-wide `IntoModel<T>` story.
pub trait IntoTableStateModel {
    fn into_table_state_model(self) -> Model<TableState>;
}

impl IntoTableStateModel for Model<TableState> {
    fn into_table_state_model(self) -> Model<TableState> {
        self
    }
}

impl IntoTableStateModel for &Model<TableState> {
    fn into_table_state_model(self) -> Model<TableState> {
        self.clone()
    }
}

use crate::declarative::action_hooks::ActionHooksExt;
use crate::declarative::collection_semantics::CollectionSemanticsExt as _;
use crate::declarative::model_watch::ModelWatchExt as _;
use crate::ui;
use crate::{IntoUiElement, LayoutRefinement, MetricRef, Size, Space, collect_children};

use crate::headless::table::{
    Aggregation, ColumnDef, ColumnId, ColumnResizeDirection, ColumnResizeMode, ExpandingState,
    FilteringFnSpec, FlatRowOrderCache, FlatRowOrderDeps, GroupedColumnMode, GroupedRowKind,
    PaginationBounds, PaginationState, Row, RowId, RowKey, SortSpec, SortToggleColumn, Table,
    TableOptions, TableState, TanStackValue, begin_column_resize, compute_grouped_u64_aggregations,
    drag_column_resize, end_column_resize, is_row_expanded, is_row_selected, is_some_rows_pinned,
    pagination_bounds, sort_grouped_row_indices_in_place, toggle_sorting_state_handler_tanstack,
};
use crate::headless::typeahead::{TypeaheadBuffer, match_prefix_arc_str};

const TABLE_TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(750);

fn resolve_table_colors(theme: &Theme) -> (Color, Color, Color, Color, Color) {
    let table_bg = theme
        .color_by_key("table.background")
        .or_else(|| theme.color_by_key("list.background"))
        .or_else(|| theme.color_by_key("card"))
        .unwrap_or_else(|| theme.color_token("card"));
    let border = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .or_else(|| theme.color_by_key("list.border"))
        .unwrap_or_else(|| theme.color_token("border"));
    let header_bg = theme
        .color_by_key("table.header.background")
        .or_else(|| theme.color_by_key("muted"))
        .unwrap_or(table_bg);
    let row_hover = theme
        .color_by_key("table.row.hover")
        .or_else(|| theme.color_by_key("list.hover.background"))
        .or_else(|| theme.color_by_key("list.row.hover"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let row_active = theme
        .color_by_key("table.row.active")
        .or_else(|| theme.color_by_key("list.active.background"))
        .or_else(|| theme.color_by_key("list.row.active"))
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
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

#[cfg(test)]
fn next_sort_for_column(current: Option<bool>) -> Option<bool> {
    match current {
        None => Some(false),
        Some(false) => Some(true),
        Some(true) => None,
    }
}

#[cfg(test)]
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

fn with_table_view_column_constraints<TData>(
    mut col: ColumnDef<TData>,
    props: &TableViewProps,
) -> ColumnDef<TData> {
    let min_w = col.min_size.max(props.min_column_width.0).max(0.0);
    col.min_size = min_w;
    col.max_size = col.max_size.max(min_w);
    if !col.columns.is_empty() {
        col.columns = col
            .columns
            .into_iter()
            .map(|c| with_table_view_column_constraints(c, props))
            .collect();
    }
    col
}

fn retained_table_row_fill_layout() -> LayoutStyle {
    LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Fill,
            ..Default::default()
        },
        flex: fret_ui::element::FlexItemStyle {
            grow: 1.0,
            shrink: 1.0,
            basis: Length::Px(Px(0.0)),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn table_scroll_fill_layout() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.size.height = Length::Fill;
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));
    layout
}

fn table_clip_fill_layout() -> LayoutStyle {
    let mut layout = table_scroll_fill_layout();
    layout.overflow = Overflow::Clip;
    layout
}

fn table_fixed_column_layout(col_w: Px) -> LayoutStyle {
    LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Px(col_w),
            min_width: Some(Length::Px(col_w)),
            max_width: Some(Length::Px(col_w)),
            ..Default::default()
        },
        flex: fret_ui::element::FlexItemStyle {
            shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn table_fixed_column_fill_layout(col_w: Px) -> LayoutStyle {
    let mut layout = table_fixed_column_layout(col_w);
    layout.size.height = Length::Fill;
    layout
}

fn table_fixed_column_clip_fill_layout(col_w: Px) -> LayoutStyle {
    let mut layout = table_fixed_column_fill_layout(col_w);
    layout.overflow = Overflow::Clip;
    layout
}

fn table_wrap_horizontal_scroll<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    scroll_handle: Option<ScrollHandle>,
    row: AnyElement,
) -> AnyElement {
    if let Some(scroll_handle) = scroll_handle {
        cx.scroll(
            ScrollProps {
                axis: ScrollAxis::X,
                scroll_handle: Some(scroll_handle),
                layout: table_scroll_fill_layout(),
                ..Default::default()
            },
            |_| vec![row],
        )
    } else {
        row
    }
}

fn take_single_root_test_id(children: &mut [AnyElement]) -> Option<Arc<str>> {
    let mut found: Option<(usize, Arc<str>)> = None;

    for (idx, child) in children.iter().enumerate() {
        let Some(test_id) = child
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.test_id.as_ref())
            .cloned()
        else {
            continue;
        };

        if found.is_some() {
            return None;
        }

        found = Some((idx, test_id));
    }

    let (idx, test_id) = found?;
    if let Some(decoration) = children[idx].semantics_decoration.as_mut() {
        decoration.test_id = None;
    }

    Some(test_id)
}

fn table_wrapper_test_id(
    children: &mut [AnyElement],
    explicit: Option<Arc<str>>,
) -> Option<Arc<str>> {
    explicit.or_else(|| take_single_root_test_id(children))
}

#[allow(clippy::too_many_arguments)]
fn retained_table_render_row_visuals<H: UiHost + 'static, TData: 'static>(
    cx: &mut ElementContext<'_, H>,
    data: Arc<[TData]>,
    data_index: usize,
    row_key: RowKey,
    bg: Option<Color>,
    props: TableViewProps,
    border: Color,
    cell_px: Px,
    cell_py: Px,
    key_handler: fret_ui::action::OnKeyDown,
    columns: Arc<[ColumnDef<TData>]>,
    col_widths: Arc<[Px]>,
    cell_at: Arc<CellAt<H, TData>>,
    row_cell_test_id_prefix: Option<Arc<str>>,
    left_col_indices: Arc<[usize]>,
    center_col_indices: Arc<[usize]>,
    right_col_indices: Arc<[usize]>,
    scroll_x: ScrollHandle,
) -> AnyElement {
    cx.key_on_key_down_for(cx.root_id(), key_handler);

    cx.container(
        ContainerProps {
            background: bg,
            layout: retained_table_row_fill_layout(),
            ..Default::default()
        },
        move |cx| {
            let render_row_group =
                |cx: &mut ElementContext<'_, H>,
                 col_indices: &[usize],
                 scroll_x_for_group: Option<ScrollHandle>| {
                    let columns = columns.clone();
                    let col_widths = col_widths.clone();
                    let cell_at = cell_at.clone();
                    let row_cell_test_id_prefix = row_cell_test_id_prefix.clone();
                    let data = data.clone();
                    let props = props.clone();

                    let row = ui::h_row(move |cx| {
                        let original = &data[data_index];

                        col_indices
                            .iter()
                            .map(|col_idx| {
                                let col = &columns[*col_idx];
                                let col_w = col_widths[*col_idx];
                                let cell = (cell_at)(cx, col, original);

                                let cell_test_id = row_cell_test_id_prefix.as_ref().map(|prefix| {
                                    Arc::<str>::from(format!(
                                        "{prefix}{row}-cell-{col}",
                                        row = row_key.0,
                                        col = col.id.as_ref()
                                    ))
                                });

                                let cell = cx.container(
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
                                        padding: Edges::symmetric(cell_px, cell_py).into(),
                                        layout: table_fixed_column_layout(col_w),
                                        ..Default::default()
                                    },
                                    move |_cx| vec![cell],
                                );

                                if let Some(test_id) = cell_test_id {
                                    cx.semantics(
                                        SemanticsProps {
                                            test_id: Some(test_id),
                                            ..Default::default()
                                        },
                                        move |_cx| vec![cell],
                                    )
                                } else {
                                    cell
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .gap(Space::N0)
                    .justify_start()
                    .items_center()
                    .into_element(cx);

                    table_wrap_horizontal_scroll(cx, scroll_x_for_group, row)
                };

            let left = render_row_group(cx, left_col_indices.as_ref(), None);
            let center = render_row_group(cx, center_col_indices.as_ref(), Some(scroll_x.clone()));
            let right = render_row_group(cx, right_col_indices.as_ref(), None);

            vec![
                ui::h_row(|_cx| [left, center, right])
                    .gap(Space::N0)
                    .justify_start()
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full())
                    .into_element(cx),
            ]
        },
    )
}

#[derive(Debug, Clone)]
pub struct TableViewProps {
    pub size: Size,
    pub row_height: Option<Px>,
    /// Optional fixed header height (defaults to `row_height` when unset).
    ///
    /// This enables shadcn-style tables where the header row height differs from body row height.
    pub header_height: Option<Px>,
    /// Controls whether the virtualized body rows are treated as fixed-height or measured.
    ///
    /// - `Fixed` (default): fast path; row containers are forced to `row_height` and the virtualizer
    ///   skips per-row measurement work.
    /// - `Measured`: enables variable-height rows (e.g. wrapping Markdown) by letting the runtime
    ///   measure visible rows and write sizes back into the virtualizer.
    pub row_measure_mode: TableRowMeasureMode,
    pub overscan: usize,
    /// Optional retained-subtree budget for overscan window shifts (retained host path).
    ///
    /// When `None`, the default heuristic is `overscan * 2`.
    ///
    /// Larger values reduce remount/layout churn when the window oscillates across boundaries
    /// (e.g. scroll bounce patterns), at the cost of retaining more offscreen subtrees.
    pub keep_alive: Option<usize>,
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
    /// When `true` (default), pointer-activating a row toggles its selection state.
    ///
    /// Set this to `false` for shadcn-style recipes where selection is driven by an explicit
    /// checkbox column (and row clicks should not toggle selection).
    pub pointer_row_selection: bool,
    /// Pointer selection policy (when `pointer_row_selection` is enabled).
    pub pointer_row_selection_policy: PointerRowSelectionPolicy,
    /// When `false`, pinned rows are only rendered if they are part of the current
    /// filtered/sorted/paginated row model (TanStack `keepPinnedRows=false`).
    ///
    /// When `true` (default), pinned rows can remain visible even when they are outside the
    /// current row model (TanStack default).
    pub keep_pinned_rows: bool,
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
    /// Grouped-mode row pinning display policy.
    ///
    /// TanStack `table-core` exposes pinning via `getTopRows/getCenterRows/getBottomRows`, and the
    /// most common UI recipe is to render pinned rows in dedicated top/bottom bands (removing them
    /// from the paged center rows). `PromotePinnedRows` matches that TanStack-typical behavior and
    /// is the default.
    pub grouped_row_pinning_policy: GroupedRowPinningPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableRowMeasureMode {
    /// Fixed-height body rows (fast path).
    #[default]
    Fixed,
    /// Variable-height body rows (measurement + write-back).
    Measured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PointerRowSelectionPolicy {
    /// Legacy behavior: a row click toggles membership in `row_selection`.
    #[default]
    Toggle,
    /// List-like behavior:
    /// - no modifiers: exclusive selection (clears then selects)
    /// - Ctrl/Meta: additive toggle
    /// - Shift: range selection from anchor
    ListLike,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroupedRowPinningPolicy {
    /// Promote pinned rows into top/bottom bands and remove duplicates from the paged center rows.
    #[default]
    PromotePinnedRows,
    /// Keep pinned leaf rows inside their grouped hierarchy and keep the paged center rows
    /// unchanged (no promotion).
    PreserveHierarchy,
}

impl Default for TableViewProps {
    fn default() -> Self {
        Self {
            size: Size::Medium,
            row_height: None,
            header_height: None,
            row_measure_mode: TableRowMeasureMode::Fixed,
            overscan: 2,
            keep_alive: None,
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
            pointer_row_selection: true,
            pointer_row_selection_policy: PointerRowSelectionPolicy::Toggle,
            keep_pinned_rows: true,
            draw_frame: true,
            optimize_paint_order: false,
            optimize_grid_lines: false,
            grouped_row_pinning_policy: GroupedRowPinningPolicy::PromotePinnedRows,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableViewOutput {
    /// Total row count after filters (and grouping expansion), before pagination is applied.
    pub filtered_row_count: usize,
    pub pagination: PaginationBounds,
}

/// Debug/test-only anchors for virtualized table harnesses.
///
/// These ids are intended for scripted diagnostics and geometry assertions:
/// - `header_row_test_id` targets the fixed header viewport row.
/// - `header_cell_test_id_prefix` targets table-owned header cell layout wrappers.
/// - `row_test_id_prefix` targets table-owned body row / cell layout wrappers.
#[derive(Debug, Clone, Default)]
pub struct TableDebugIds {
    pub header_row_test_id: Option<Arc<str>>,
    pub header_cell_test_id_prefix: Option<Arc<str>>,
    pub row_test_id_prefix: Option<Arc<str>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = include_str!("table.rs");
    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, PathCommand, PointerEvent, PointerId,
        PointerType, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics,
        TextService,
    };
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect, TextWrap};
    use fret_ui::ThemeConfig;
    use fret_ui::{Theme, UiTree, VirtualListScrollHandle};

    #[test]
    fn table_surfaces_keep_a_narrow_table_state_bridge() {
        assert!(
            SOURCE.contains("pub trait IntoTableStateModel {"),
            "table surfaces should keep a dedicated TableState bridge instead of widening into a generic model conversion story"
        );
        assert!(
            SOURCE.contains("pub fn table_virtualized<H: UiHost, TData, IHeader, TH, ICell, TC>(")
                && SOURCE.contains("state: impl IntoTableStateModel,"),
            "table_virtualized should accept the dedicated table-state bridge"
        );
        assert!(
            SOURCE.contains("pub fn table_virtualized_retained_v0<H: UiHost + 'static, TData>("),
            "retained table surface should accept the dedicated table-state bridge"
        );
        assert!(
            !SOURCE.contains(
                "pub fn table_virtualized<H: UiHost, TData, IHeader, TH, ICell, TC>(\n    cx: &mut ElementContext<'_, H>,\n    data: &[TData],\n    columns: &[ColumnDef<TData>],\n    state: Model<TableState>,"
            ),
            "table_virtualized should not regress to a raw Model<TableState>-only signature"
        );
    }

    #[test]
    fn retained_table_callbacks_prefer_explicit_context_access_capability() {
        assert!(
            SOURCE.contains("dyn for<'a> Fn(&mut dyn ElementContextAccess<'a, H>, &ColumnDef<TData>) -> AnyElement;")
                || SOURCE.contains(
                    "dyn for<'a> Fn(\n        &mut dyn ElementContextAccess<'a, H>,\n        &ColumnDef<TData>,\n    ) -> AnyElement;"
                ),
            "retained table header accessories should accept explicit context access capability"
        );
        assert!(
            SOURCE.contains("dyn for<'a> Fn(&mut dyn ElementContextAccess<'a, H>, &ColumnDef<TData>, &TData) -> AnyElement;")
                || SOURCE.contains(
                    "dyn for<'a> Fn(\n        &mut dyn ElementContextAccess<'a, H>,\n        &ColumnDef<TData>,\n        &TData,\n    ) -> AnyElement;"
                ),
            "retained table cell renderers should accept explicit context access capability"
        );
    }

    #[test]
    fn table_debug_ids_expose_explicit_header_row_anchor() {
        assert!(
            SOURCE.contains("pub struct TableDebugIds {"),
            "table harnesses should use a structured debug-id surface"
        );
        assert!(
            SOURCE.contains("pub header_row_test_id: Option<Arc<str>>,"),
            "table debug ids should expose an explicit header-row anchor"
        );
        assert!(
            SOURCE.contains("debug_ids: TableDebugIds,"),
            "table surfaces should accept a shared structured debug-id contract"
        );
    }

    #[test]
    fn table_virtualized_hoists_single_root_renderer_test_ids_to_layout_anchors() {
        assert!(
            SOURCE.contains(
                "fn take_single_root_test_id(children: &mut [AnyElement]) -> Option<Arc<str>> {"
            ),
            "table_virtualized should keep the single-root test-id hoist helper close to the table surface"
        );
        assert!(
            SOURCE.contains(
                "fn table_wrapper_test_id(\n    children: &mut [AnyElement],\n    explicit: Option<Arc<str>>,\n) -> Option<Arc<str>> {"
            ),
            "table_virtualized should resolve explicit debug ids before falling back to renderer-root hoists"
        );
        assert!(
            SOURCE.match_indices("table_wrapper_test_id(").count() >= 4,
            "table_virtualized should hoist marked header/cell renderer roots onto stable layout anchors"
        );
    }

    #[test]
    fn table_virtualized_retained_accepts_capability_first_cell_renderer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 1;
        let state = app.models_mut().insert(state_value);

        let data: Arc<[u32]> = Arc::from(vec![0u32]);
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![{
            let mut col = ColumnDef::new("name");
            col.size = 220.0;
            col
        }]);
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    TableViewProps::default(),
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(retained_table_capability_test_cell),
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "table-retained-capability-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from(
                            "table-retained-capability-row-",
                        )),
                        ..Default::default()
                    },
                )]
            })
        };

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after retained table render");

        assert!(
            snap.nodes.iter().any(
                |node| node.test_id.as_deref() == Some("table-retained-capability-cell-name-0")
            ),
            "expected retained table capability-first cell renderer to contribute its semantics marker"
        );
    }

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

    #[test]
    fn collect_leaf_keys_skips_group_rows() {
        let meta = vec![
            TableNavRowMeta {
                row_key: RowKey(1),
                kind: TableNavRowKind::Leaf,
                data_index: Some(0),
                label: Arc::from("Alpha"),
            },
            TableNavRowMeta {
                row_key: RowKey(2),
                kind: TableNavRowKind::Group,
                data_index: None,
                label: Arc::from("Group"),
            },
            TableNavRowMeta {
                row_key: RowKey(3),
                kind: TableNavRowKind::Leaf,
                data_index: Some(1),
                label: Arc::from("Beta"),
            },
        ];

        assert_eq!(table_collect_leaf_keys(&meta), vec![RowKey(1), RowKey(3)]);
        assert_eq!(
            table_collect_leaf_keys_in_range(&meta, 0, 2),
            vec![RowKey(1), RowKey(3)]
        );
        assert_eq!(
            table_collect_leaf_keys_in_range(&meta, 1, 1),
            Vec::<RowKey>::new()
        );
        assert_eq!(
            table_collect_leaf_keys_in_range(&meta, 99, 100),
            vec![RowKey(3)]
        );
    }

    #[test]
    fn apply_row_pinning_to_paged_rows_surfaces_pinned_outside_page_and_dedupes() {
        let visible_all = vec![
            DisplayRow::Leaf {
                data_index: 0,
                row_key: RowKey(1),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 1,
                row_key: RowKey(2),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 2,
                row_key: RowKey(3),
                depth: 0,
            },
        ];

        let page_rows = vec![visible_all[1].clone(), visible_all[2].clone()];

        let row_pinning = crate::headless::table::RowPinningState {
            top: vec![RowKey(1)],
            bottom: vec![RowKey(3)],
        };

        let out = apply_row_pinning_to_paged_rows(&visible_all, &page_rows, &row_pinning);
        let keys = out.into_iter().map(|r| r.row_key()).collect::<Vec<_>>();
        assert_eq!(keys, vec![RowKey(1), RowKey(2), RowKey(3)]);
    }

    #[test]
    fn grouped_row_pinning_policy_preserve_hierarchy_keeps_page_rows_center_unchanged() {
        let visible_all = vec![
            DisplayRow::Leaf {
                data_index: 0,
                row_key: RowKey(1),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 1,
                row_key: RowKey(2),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 2,
                row_key: RowKey(3),
                depth: 0,
            },
        ];

        let page_rows_center = vec![visible_all[1].clone(), visible_all[2].clone()];
        let row_pinning = crate::headless::table::RowPinningState {
            top: vec![RowKey(1)],
            bottom: vec![RowKey(3)],
        };

        let out = apply_grouped_row_pinning_policy(
            &visible_all,
            &page_rows_center,
            &row_pinning,
            GroupedRowPinningPolicy::PreserveHierarchy,
        );
        let keys = out.into_iter().map(|r| r.row_key()).collect::<Vec<_>>();
        assert_eq!(keys, vec![RowKey(2), RowKey(3)]);
    }

    #[test]
    fn grouped_row_pinning_policy_promote_pinned_rows_matches_legacy_behavior() {
        let visible_all = vec![
            DisplayRow::Leaf {
                data_index: 0,
                row_key: RowKey(1),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 1,
                row_key: RowKey(2),
                depth: 0,
            },
            DisplayRow::Leaf {
                data_index: 2,
                row_key: RowKey(3),
                depth: 0,
            },
        ];

        let page_rows_center = vec![visible_all[1].clone(), visible_all[2].clone()];
        let row_pinning = crate::headless::table::RowPinningState {
            top: vec![RowKey(1)],
            bottom: vec![RowKey(3)],
        };

        let out = apply_grouped_row_pinning_policy(
            &visible_all,
            &page_rows_center,
            &row_pinning,
            GroupedRowPinningPolicy::PromotePinnedRows,
        );
        let keys = out.into_iter().map(|r| r.row_key()).collect::<Vec<_>>();
        assert_eq!(keys, vec![RowKey(1), RowKey(2), RowKey(3)]);
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Err(fret_core::MaterialRegistrationError::Unsupported)
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

    fn retained_table_capability_test_cell<'a>(
        cx: &mut dyn ElementContextAccess<'a, App>,
        col: &ColumnDef<u32>,
        row: &u32,
    ) -> AnyElement {
        let cx = cx.elements();
        let label = format!("{}-{row}", col.id.as_ref());
        let test_id = Arc::<str>::from(format!("table-retained-capability-cell-{label}"));
        cx.semantics(
            SemanticsProps {
                test_id: Some(test_id),
                ..Default::default()
            },
            move |cx| vec![cx.text(label.clone())],
        )
    }

    fn capture_layout_sidecar(
        ui: &UiTree<App>,
        app: &mut App,
        window: AppWindowId,
        root: fret_core::NodeId,
        bounds: Rect,
    ) -> serde_json::Value {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let out_dir = std::env::temp_dir().join(format!(
            "fret-ui-kit-table-layout-sidecar-{}-{nonce}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&out_dir);

        let path = ui
            .debug_write_layout_sidecar_taffy_v1_json(
                app, window, root, bounds, 1.0, None, &out_dir, 0,
            )
            .expect("layout sidecar should be written");

        let sidecar: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).expect("sidecar should be readable"))
                .expect("sidecar json should parse");
        let _ = std::fs::remove_dir_all(&out_dir);
        sidecar
    }

    fn layout_sidecar_abs_rect(sidecar: &serde_json::Value, test_id: &str) -> Rect {
        let mut candidates: Vec<&serde_json::Value> = sidecar["taffy"]["nodes"]
            .as_array()
            .into_iter()
            .flat_map(|nodes| nodes.iter())
            .collect();
        if let Some(roots) = sidecar["taffy"]["roots"].as_array() {
            for root in roots {
                if let Some(nodes) = root["dump"]["nodes"].as_array() {
                    candidates.extend(nodes.iter());
                }
            }
        }

        let matches = candidates
            .iter()
            .filter(|node| node["debug"]["test_id"].as_str() == Some(test_id))
            .copied()
            .collect::<Vec<_>>();
        let matched = matches
            .into_iter()
            .max_by(|a, b| {
                let area = |node: &serde_json::Value| {
                    let rect = &node["abs_rect"];
                    rect["w"].as_f64().unwrap_or(0.0) * rect["h"].as_f64().unwrap_or(0.0)
                };
                area(a)
                    .partial_cmp(&area(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|| {
                let available = candidates
                    .iter()
                    .filter_map(|node| node["debug"]["test_id"].as_str())
                    .collect::<Vec<_>>();
                panic!(
                    "expected layout sidecar node with test_id `{test_id}`; available_test_ids={available:?}"
                );
            });
        let abs = &matched["abs_rect"];
        Rect::new(
            Point::new(
                Px(abs["x"].as_f64().expect("abs_rect.x should be a number") as f32),
                Px(abs["y"].as_f64().expect("abs_rect.y should be a number") as f32),
            ),
            fret_core::Size::new(
                Px(abs["w"].as_f64().expect("abs_rect.w should be a number") as f32),
                Px(abs["h"].as_f64().expect("abs_rect.h should be a number") as f32),
            ),
        )
    }

    #[test]
    fn table_virtualized_copyable_reports_availability_and_emits_clipboard_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut caps = fret_runtime::PlatformCapabilities::default();
        caps.clipboard.text.read = true;
        caps.clipboard.text.write = true;
        app.set_global(caps);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let state = app.models_mut().insert(TableState::default());
        let data = vec![0u32, 1u32, 2u32];
        let columns = vec![ColumnDef::new("col")];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_copyable(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    TableViewProps::default(),
                    Arc::new(|_models, i| Some(format!("Row {i}"))),
                    |_row| None,
                    |cx, _col, _sort| [cx.text("Header")],
                    |cx, row, _col| [cx.text(format!("Cell {}", row.index))],
                    None,
                    TableDebugIds::default(),
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        let mut root = fret_core::NodeId::default();
        for _ in 0..2 {
            root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let table_node = ui.children(root)[0];
        ui.set_focus(Some(table_node));

        let copy = CommandId::from("edit.copy");
        assert!(
            !ui.is_command_available(&mut app, &copy),
            "expected edit.copy to be unavailable when selection is empty"
        );
        assert!(
            ui.dispatch_command(&mut app, &mut services, &copy),
            "expected edit.copy to be handled by the table surface"
        );
        let effects = app.flush_effects();
        assert!(
            !effects
                .iter()
                .any(|e| matches!(e, fret_runtime::Effect::ClipboardWriteText { .. })),
            "expected edit.copy to not emit ClipboardWriteText when selection is empty"
        );

        let _ = app.models_mut().update(&state, |st| {
            st.row_selection.insert(RowKey::from_index(1));
        });

        assert!(
            ui.is_command_available(&mut app, &copy),
            "expected edit.copy to be available when selection is non-empty"
        );
        assert!(
            ui.dispatch_command(&mut app, &mut services, &copy),
            "expected edit.copy to be handled by the table surface"
        );
        let effects = app.flush_effects();
        assert!(
            effects.iter().any(|e| {
                matches!(e, fret_runtime::Effect::ClipboardWriteText { text, .. } if text == "Row 1")
            }),
            "expected edit.copy to emit ClipboardWriteText for the selected row"
        );
    }

    #[test]
    fn table_virtualized_clamps_cell_width_for_wide_text() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 1;
        let state = app.models_mut().insert(state_value);

        let data = vec![0u32];
        let mut col = ColumnDef::new("col");
        col.size = 80.0;
        let columns = vec![col];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    TableViewProps::default(),
                    |_row| None,
                    move |cx, _col, _sort| {
                        let header = cx.semantics(
                            SemanticsProps {
                                test_id: Some(Arc::<str>::from("table-test-header")),
                                ..Default::default()
                            },
                            |cx| {
                                vec![cx.container(
                                    ContainerProps {
                                        layout: LayoutStyle {
                                            size: fret_ui::element::SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Fill,
                                                ..Default::default()
                                            },
                                            overflow: Overflow::Clip,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |cx| vec![crate::ui::text("Header").into_element(cx)],
                                )]
                            },
                        );
                        [header]
                    },
                    |cx, row, _col| {
                        let long = format!("Row{}-{}", row.index, "x".repeat(4096));
                        let cell = crate::ui::text(long).wrap(TextWrap::Grapheme);
                        let cell = cx.container(
                            ContainerProps {
                                layout: LayoutStyle {
                                    size: fret_ui::element::SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    overflow: Overflow::Clip,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |cx| vec![cell.into_element(cx)],
                        );

                        [cx.semantics(
                            SemanticsProps {
                                test_id: Some(Arc::<str>::from("table-test-cell")),
                                ..Default::default()
                            },
                            move |_cx| vec![cell],
                        )]
                    },
                    None,
                    TableDebugIds::default(),
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        for _ in 0..3 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected a semantics snapshot");

        let cell_bounds = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("table-test-cell"))
            .map(|n| n.bounds)
            .expect("expected to find table-test-cell");

        assert!(
            cell_bounds.size.width.0 <= 80.0,
            "expected the cell subtree to be clamped to the column width (got {:.2}px)",
            cell_bounds.size.width.0
        );
    }

    #[test]
    fn table_virtualized_alignment_gate_header_matches_rows_under_overflow_and_variable_height() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 3;
        let state = app.models_mut().insert(state_value);

        let data = vec![0u32, 1u32, 2u32];
        let columns = vec![
            {
                let mut col = ColumnDef::new("name");
                col.size = 220.0;
                col
            },
            {
                let mut col = ColumnDef::new("status");
                col.size = 140.0;
                col
            },
            {
                let mut col = ColumnDef::new("cpu%");
                col.size = 90.0;
                col
            },
            {
                let mut col = ColumnDef::new("mem_mb");
                col.size = 110.0;
                col
            },
        ];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            row_measure_mode: TableRowMeasureMode::Measured,
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    props.clone(),
                    |_row| None,
                    |cx, col, _sort| {
                        let label = Arc::<str>::from(col.id.as_ref());
                        [cx.container(
                            ContainerProps {
                                layout: table_clip_fill_layout(),
                                ..Default::default()
                            },
                            move |_cx| vec![crate::ui::text(label.clone()).into_element(_cx)],
                        )]
                    },
                    |cx, row, col| {
                        let text = match col.id.as_ref() {
                            "name" => {
                                if row.index == 1 {
                                    format!("Row {} (details)\nMore text to force wrap", row.index)
                                } else {
                                    format!("Row {}", row.index)
                                }
                            }
                            "status" => "Running".to_string(),
                            "cpu%" => "42%".to_string(),
                            "mem_mb" => "256 MB".to_string(),
                            _ => "?".to_string(),
                        };
                        let cell = crate::ui::text(text).wrap(TextWrap::Grapheme);
                        [cx.container(
                            ContainerProps {
                                layout: table_clip_fill_layout(),
                                ..Default::default()
                            },
                            move |_cx| vec![cell.clone().into_element(_cx)],
                        )]
                    },
                    None,
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from("table-align-header-")),
                        row_test_id_prefix: Some(Arc::<str>::from("table-align-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        let mut root = fret_core::NodeId::default();
        for _ in 0..2 {
            root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after table render");
        let col_ids = ["name", "status", "cpu%", "mem_mb"];
        for col_id in col_ids {
            let header_test_id = format!("table-align-header-{col_id}");
            let header_count = snap
                .nodes
                .iter()
                .filter(|node| node.test_id.as_deref() == Some(header_test_id.as_str()))
                .count();
            assert_eq!(
                header_count, 1,
                "expected exactly one semantics node for hoisted header anchor {header_test_id}"
            );

            for row in 0..3 {
                let cell_test_id = format!("table-align-row-{row}-cell-{col_id}");
                let cell_count = snap
                    .nodes
                    .iter()
                    .filter(|node| node.test_id.as_deref() == Some(cell_test_id.as_str()))
                    .count();
                assert_eq!(
                    cell_count, 1,
                    "expected exactly one semantics node for hoisted cell anchor {cell_test_id}"
                );
            }
        }

        let sidecar = capture_layout_sidecar(&ui, &mut app, window, root, bounds);

        // `table_virtualized` composes header/body content under slightly different chrome
        // (e.g. grid line dividers vs resize handles). We want a strict gate for x alignment
        // (columns must not shift across rows), while allowing a small tolerance for the
        // content-box width when borders are involved.
        let eps_x = 0.5;
        let eps_w = 1.0;
        for col_id in col_ids {
            let header = layout_sidecar_abs_rect(&sidecar, &format!("table-align-header-{col_id}"));
            for row in 0..3 {
                let cell = layout_sidecar_abs_rect(
                    &sidecar,
                    &format!("table-align-row-{row}-cell-{col_id}"),
                );
                assert!(
                    (cell.origin.x.0 - header.origin.x.0).abs() <= eps_x,
                    "expected header/cell x alignment for col={col_id} row={row} (header_x={:.2}px cell_x={:.2}px)",
                    header.origin.x.0,
                    cell.origin.x.0
                );
                assert!(
                    (cell.size.width.0 - header.size.width.0).abs() <= eps_w,
                    "expected header/cell width alignment for col={col_id} row={row} (header_w={:.2}px cell_w={:.2}px)",
                    header.size.width.0,
                    cell.size.width.0
                );
            }
        }
    }

    #[test]
    fn table_virtualized_pointer_select_does_not_shift_row_bounds() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 5;
        let state = app.models_mut().insert(state_value);

        let data = vec![0u32, 1u32, 2u32, 3u32, 4u32];
        let mut col = ColumnDef::new("name");
        col.size = 220.0;
        let columns = vec![col];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            single_row_selection: true,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    props.clone(),
                    |_row| None,
                    |cx, _col, _sort| [cx.text("")],
                    |cx, row, _col| {
                        let mut fill = LayoutStyle::default();
                        fill.size.width = Length::Fill;
                        fill.size.height = Length::Fill;
                        let marker = cx.container(
                            ContainerProps {
                                layout: fill,
                                ..Default::default()
                            },
                            |_| Vec::new(),
                        );
                        let _ = row;
                        [marker]
                    },
                    None,
                    TableDebugIds {
                        row_test_id_prefix: Some(Arc::<str>::from("table-test-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected a semantics snapshot");
        let before = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("table-test-row-1"))
            .map(|n| n.bounds)
            .expect("expected marker node");

        let click_pos = Point::new(
            Px(before.origin.x.0 + before.size.width.0 * 0.5),
            Px(before.origin.y.0 + before.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected a semantics snapshot");
        let after = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("table-test-row-1"))
            .map(|n| n.bounds)
            .expect("expected marker node");

        assert!(
            (after.origin.y.0 - before.origin.y.0).abs() <= 0.1,
            "expected row marker to keep stable y; before={:?} after={:?}",
            before,
            after
        );
    }

    #[test]
    fn table_virtualized_can_disable_pointer_row_selection() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 5;
        let state = app.models_mut().insert(state_value);

        let data = vec![0u32, 1u32, 2u32, 3u32, 4u32];
        let mut col = ColumnDef::new("name");
        col.size = 220.0;
        let columns = vec![col];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(200.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            pointer_row_selection: false,
            single_row_selection: true,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    props.clone(),
                    |_row| None,
                    |cx, _col, _sort| [cx.text("")],
                    |cx, row, _col| {
                        let mut fill = LayoutStyle::default();
                        fill.size.width = Length::Fill;
                        fill.size.height = Length::Fill;
                        let marker = cx.container(
                            ContainerProps {
                                layout: fill,
                                ..Default::default()
                            },
                            |_| Vec::new(),
                        );
                        let _ = row;
                        [marker]
                    },
                    None,
                    TableDebugIds {
                        row_test_id_prefix: Some(Arc::<str>::from("table-test-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected a semantics snapshot");
        let marker_bounds = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("table-test-row-1"))
            .map(|n| n.bounds)
            .expect("expected marker node");

        let click_pos = Point::new(
            Px(marker_bounds.origin.x.0 + marker_bounds.size.width.0 * 0.5),
            Px(marker_bounds.origin.y.0 + marker_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        let selection = app
            .models()
            .read(&state, |st| st.row_selection.clone())
            .ok()
            .unwrap_or_default();
        assert!(
            !selection.contains(&RowKey::from_index(1)),
            "expected row click not to toggle selection when pointer_row_selection=false"
        );
    }

    #[test]
    fn table_virtualized_nested_pressable_remains_hittable_when_pointer_row_selection_disabled() {
        use std::cell::Cell;
        use std::rc::Rc;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 3;
        let state = app.models_mut().insert(state_value);
        let child_activated = app.models_mut().insert(false);
        let child_element: Rc<Cell<Option<fret_ui::GlobalElementId>>> = Rc::new(Cell::new(None));

        let data = vec![0u32, 1u32, 2u32];
        let columns = vec![
            {
                let mut col = ColumnDef::new("name");
                col.size = 180.0;
                col
            },
            {
                let mut col = ColumnDef::new("actions");
                col.size = 80.0;
                col
            },
        ];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            pointer_row_selection: false,
            single_row_selection: true,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            let child_element = child_element.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    props.clone(),
                    |_row| None,
                    |cx, col, _sort| [cx.text(col.id.as_ref())],
                    |cx, row, col| match col.id.as_ref() {
                        "name" => [cx.text(format!("Row {}", row.index))],
                        "actions" if row.index == 1 => {
                            let child_element = child_element.clone();
                            let child_activated = child_activated.clone();
                            [cx.pressable_with_id(
                                PressableProps {
                                    focusable: false,
                                    layout: LayoutStyle {
                                        size: fret_ui::element::SizeStyle {
                                            width: Length::Px(Px(24.0)),
                                            height: Length::Px(Px(24.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        test_id: Some(Arc::<str>::from("table-test-child-button")),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |cx, _st, id| {
                                    child_element.set(Some(id));
                                    let child_activated = child_activated.clone();
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, _acx, _reason| {
                                            let _ = host
                                                .models_mut()
                                                .update(&child_activated, |value| *value = true);
                                        },
                                    ));
                                    vec![cx.spacer(SpacerProps::default())]
                                },
                            )]
                        }
                        "actions" => [cx.text("-")],
                        _ => [cx.text("?")],
                    },
                    None,
                    TableDebugIds::default(),
                )]
            })
        };

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let child_element = child_element
            .get()
            .expect("expected nested child pressable element");
        let child_node = fret_ui::elements::node_for_element(&mut app, window, child_element)
            .expect("expected nested child pressable node");
        let child_bounds = ui
            .debug_node_bounds(child_node)
            .expect("expected nested child bounds");
        assert!(
            child_bounds.size.width.0 > 0.0 && child_bounds.size.height.0 > 0.0,
            "expected nested child pressable to have non-zero bounds, got {child_bounds:?}"
        );
        let click_pos = Point::new(
            Px(child_bounds.origin.x.0 + child_bounds.size.width.0 * 0.5),
            Px(child_bounds.origin.y.0 + child_bounds.size.height.0 * 0.5),
        );

        let hit = ui.debug_hit_test_routing(click_pos);
        let hit_node = hit.hit.expect("expected nested child hit");
        let path = ui.debug_node_path(hit_node);
        let child_path = ui.debug_node_path(child_node);
        let child_path_debug = child_path
            .iter()
            .map(|node| {
                (
                    *node,
                    ui.debug_node_bounds(*node),
                    ui.debug_node_clips_hit_test(*node),
                    ui.debug_node_can_scroll_descendant_into_view(*node),
                )
            })
            .collect::<Vec<_>>();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        assert_eq!(
            app.models().get_copied(&child_activated),
            Some(true),
            "expected nested child pressable to activate; hit={hit:?} path={path:?} child={child_node:?} child_path={child_path:?} child_path_debug={child_path_debug:?} child_bounds={child_bounds:?}"
        );

        let selection = app
            .models()
            .read(&state, |st| st.row_selection.clone())
            .ok()
            .unwrap_or_default();
        assert!(
            selection.is_empty(),
            "expected nested child pressable click not to toggle row selection when pointer_row_selection=false"
        );
    }

    #[test]
    fn table_virtualized_pointer_row_selection_policy_list_like() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 5;
        let state = app.models_mut().insert(state_value);

        let data = vec![0u32, 1u32, 2u32, 3u32, 4u32];
        let mut col = ColumnDef::new("name");
        col.size = 220.0;
        let columns = vec![col];
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            pointer_row_selection: true,
            pointer_row_selection_policy: PointerRowSelectionPolicy::ListLike,
            single_row_selection: false,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized(
                    cx,
                    &data,
                    &columns,
                    state.clone(),
                    &scroll,
                    0,
                    &|_row, i| RowKey::from_index(i),
                    None,
                    props.clone(),
                    |_row| None,
                    |cx, _col, _sort| [cx.text("")],
                    |cx, row, _col| {
                        let mut fill = LayoutStyle::default();
                        fill.size.width = Length::Fill;
                        fill.size.height = Length::Fill;
                        let marker = cx.container(
                            ContainerProps {
                                layout: fill,
                                ..Default::default()
                            },
                            |_| Vec::new(),
                        );
                        let _ = row;
                        [marker]
                    },
                    None,
                    TableDebugIds {
                        row_test_id_prefix: Some(Arc::<str>::from("table-test-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let click_pos_for_row: Vec<Point> = {
            let snap = ui
                .semantics_snapshot()
                .expect("expected a semantics snapshot");
            (0..5)
                .map(|row_index| {
                    let id = format!("table-test-row-{row_index}");
                    let bounds = snap
                        .nodes
                        .iter()
                        .find(|n| n.test_id.as_deref() == Some(id.as_str()))
                        .map(|n| n.bounds)
                        .unwrap_or_else(|| panic!("expected marker node {id}"));
                    Point::new(
                        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                    )
                })
                .collect()
        };

        let click_row = |ui: &mut UiTree<App>,
                         app: &mut App,
                         services: &mut FakeServices,
                         row_index: usize,
                         modifiers: Modifiers| {
            let click_pos = click_pos_for_row[row_index];
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    position: click_pos,
                    button: MouseButton::Left,
                    modifiers,
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    position: click_pos,
                    button: MouseButton::Left,
                    modifiers,
                    click_count: 1,
                    is_click: true,
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
        };

        let assert_selected = |app: &App, expected: &[RowKey]| {
            let selection = app
                .models()
                .read(&state, |st| st.row_selection.clone())
                .ok()
                .unwrap_or_default();
            assert_eq!(
                selection.len(),
                expected.len(),
                "expected selection len {} but got {}",
                expected.len(),
                selection.len()
            );
            for k in expected {
                assert!(selection.contains(k), "expected selection to contain {k:?}");
            }
        };

        click_row(&mut ui, &mut app, &mut services, 1, Modifiers::default());
        assert_selected(&app, &[RowKey::from_index(1)]);

        click_row(
            &mut ui,
            &mut app,
            &mut services,
            3,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_selected(
            &app,
            &[
                RowKey::from_index(1),
                RowKey::from_index(2),
                RowKey::from_index(3),
            ],
        );

        click_row(
            &mut ui,
            &mut app,
            &mut services,
            4,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        assert_selected(
            &app,
            &[
                RowKey::from_index(1),
                RowKey::from_index(2),
                RowKey::from_index(3),
                RowKey::from_index(4),
            ],
        );

        click_row(&mut ui, &mut app, &mut services, 2, Modifiers::default());
        assert_selected(&app, &[RowKey::from_index(2)]);
    }

    #[test]
    fn table_virtualized_allows_header_height_override() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 1;
        let state = app.models_mut().insert(state_value);

        let data: Arc<[u32]> = Arc::from(vec![0u32]);
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![{
            let mut col = ColumnDef::new("name");
            col.size = 220.0;
            col
        }]);
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            row_height: Some(Px(36.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props.clone(),
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        |cx: &mut dyn ElementContextAccess<'_, App>,
                         _col: &ColumnDef<u32>,
                         _row: &u32| {
                            crate::ui::text("Row 0").into_element(cx.elements())
                        },
                    ),
                    TableDebugIds {
                        header_row_test_id: Some(Arc::<str>::from("table-test-header-row")),
                        header_cell_test_id_prefix: Some(Arc::<str>::from("table-test-header-")),
                        row_test_id_prefix: Some(Arc::<str>::from("table-test-row-")),
                    },
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let snap = ui
            .semantics_snapshot()
            .expect("expected a semantics snapshot");

        let find_height = |id: &str| -> f32 {
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(id))
                .map(|n| n.bounds.size.height.0)
                .unwrap_or_else(|| panic!("expected to find {id}"))
        };

        let header_h = find_height("table-test-header-row");
        let body_h = find_height("table-test-row-0");

        let eps = 2.0;
        assert!(
            (header_h - 40.0).abs() <= eps,
            "expected header height ~40px (got {header_h:.2}px)"
        );
        assert!(
            (body_h - 36.0).abs() <= eps,
            "expected body height ~36px (got {body_h:.2}px)"
        );
    }

    #[test]
    fn table_virtualized_retained_pointer_row_selection_policy_list_like() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 5;
        let state = app.models_mut().insert(state_value);

        let data: Arc<[u32]> = Arc::from((0u32..5).collect::<Vec<_>>());
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![{
            let mut col = ColumnDef::new("name");
            col.size = 220.0;
            col
        }]);

        let scroll = VirtualListScrollHandle::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            pointer_row_selection: true,
            pointer_row_selection_policy: PointerRowSelectionPolicy::ListLike,
            single_row_selection: false,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props.clone(),
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        |cx: &mut dyn ElementContextAccess<'_, App>,
                         _col: &ColumnDef<u32>,
                         row: &u32| {
                            crate::ui::text(format!("Row {row}")).into_element(cx.elements())
                        },
                    ),
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from("table-test-header-")),
                        row_test_id_prefix: Some(Arc::<str>::from("table-test-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        // VirtualList computes the visible window based on viewport metrics populated during layout,
        // so it takes two frames for the first set of rows to mount.
        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let click_pos_for_row: Vec<Point> = {
            let snap = ui
                .semantics_snapshot()
                .expect("expected a semantics snapshot");
            (0..5)
                .map(|row_index| {
                    let id = format!("table-test-row-{row_index}");
                    let bounds = snap
                        .nodes
                        .iter()
                        .find(|n| n.test_id.as_deref() == Some(id.as_str()))
                        .map(|n| n.bounds)
                        .unwrap_or_else(|| panic!("expected row node {id}"));
                    Point::new(
                        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                    )
                })
                .collect()
        };

        let click_row = |ui: &mut UiTree<App>,
                         app: &mut App,
                         services: &mut FakeServices,
                         row_index: usize,
                         modifiers: Modifiers| {
            let click_pos = click_pos_for_row[row_index];
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Down {
                    position: click_pos,
                    button: MouseButton::Left,
                    modifiers,
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
            ui.dispatch_event(
                app,
                services,
                &Event::Pointer(PointerEvent::Up {
                    position: click_pos,
                    button: MouseButton::Left,
                    modifiers,
                    click_count: 1,
                    is_click: true,
                    pointer_id: PointerId(0),
                    pointer_type: PointerType::Mouse,
                }),
            );
        };

        let assert_selected = |app: &App, expected: &[RowKey]| {
            let selection = app
                .models()
                .read(&state, |st| st.row_selection.clone())
                .ok()
                .unwrap_or_default();
            assert_eq!(
                selection.len(),
                expected.len(),
                "expected selection len {} but got {}",
                expected.len(),
                selection.len()
            );
            for k in expected {
                assert!(selection.contains(k), "expected selection to contain {k:?}");
            }
        };

        click_row(&mut ui, &mut app, &mut services, 1, Modifiers::default());
        assert_selected(&app, &[RowKey::from_index(1)]);

        click_row(
            &mut ui,
            &mut app,
            &mut services,
            3,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_selected(
            &app,
            &[
                RowKey::from_index(1),
                RowKey::from_index(2),
                RowKey::from_index(3),
            ],
        );

        click_row(
            &mut ui,
            &mut app,
            &mut services,
            4,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        assert_selected(
            &app,
            &[
                RowKey::from_index(1),
                RowKey::from_index(2),
                RowKey::from_index(3),
                RowKey::from_index(4),
            ],
        );

        click_row(&mut ui, &mut app, &mut services, 2, Modifiers::default());
        assert_selected(&app, &[RowKey::from_index(2)]);
    }

    #[test]
    fn table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled()
     {
        use std::cell::Cell;
        use std::rc::Rc;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_debug_enabled(true);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 3;
        let state = app.models_mut().insert(state_value);
        let child_activated = app.models_mut().insert(false);
        let child_element: Rc<Cell<Option<fret_ui::GlobalElementId>>> = Rc::new(Cell::new(None));

        let data: Arc<[u32]> = Arc::from(vec![0u32, 1u32, 2u32]);
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![
            {
                let mut col = ColumnDef::new("name");
                col.size = 180.0;
                col
            },
            {
                let mut col = ColumnDef::new("actions");
                col.size = 80.0;
                col
            },
        ]);
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(180.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            enable_column_resizing: false,
            enable_row_selection: true,
            pointer_row_selection: false,
            single_row_selection: true,
            row_height: Some(Px(40.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            let child_element = child_element.clone();
            let child_activated = child_activated.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props.clone(),
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        move |cx: &mut dyn ElementContextAccess<'_, App>,
                              col: &ColumnDef<u32>,
                              row: &u32| {
                            let cx = cx.elements();
                            match col.id.as_ref() {
                                "name" => crate::ui::text(format!("Row {row}")).into_element(cx),
                                "actions" if *row == 1 => {
                                    let child_element = child_element.clone();
                                    let child_activated = child_activated.clone();
                                    cx.pressable_with_id(
                                        PressableProps {
                                            focusable: false,
                                            layout: LayoutStyle {
                                                size: fret_ui::element::SizeStyle {
                                                    width: Length::Px(Px(24.0)),
                                                    height: Length::Px(Px(24.0)),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            a11y: PressableA11y {
                                                role: Some(SemanticsRole::Button),
                                                test_id: Some(Arc::<str>::from(
                                                    "table-retained-test-child-button",
                                                )),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        move |cx, _st, id| {
                                            child_element.set(Some(id));
                                            let child_activated = child_activated.clone();
                                            cx.pressable_on_activate(Arc::new(
                                                move |host, _acx, _reason| {
                                                    let _ = host
                                                        .models_mut()
                                                        .update(&child_activated, |value| {
                                                            *value = true
                                                        });
                                                },
                                            ));
                                            vec![cx.spacer(SpacerProps::default())]
                                        },
                                    )
                                }
                                "actions" => cx.text("-"),
                                _ => cx.text("?"),
                            }
                        },
                    ),
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "table-retained-test-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from("table-retained-test-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        for _ in 0..2 {
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.request_semantics_snapshot();
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
            let mut scene = fret_core::Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        }

        let child_element = child_element
            .get()
            .expect("expected nested child pressable element");
        let child_node = fret_ui::elements::node_for_element(&mut app, window, child_element)
            .expect("expected nested child pressable node");
        let child_bounds = ui
            .debug_node_bounds(child_node)
            .expect("expected nested child bounds");
        assert!(
            child_bounds.size.width.0 > 0.0 && child_bounds.size.height.0 > 0.0,
            "expected nested child pressable to have non-zero bounds, got {child_bounds:?}"
        );
        let click_pos = Point::new(
            Px(child_bounds.origin.x.0 + child_bounds.size.width.0 * 0.5),
            Px(child_bounds.origin.y.0 + child_bounds.size.height.0 * 0.5),
        );

        let hit = ui.debug_hit_test_routing(click_pos);
        let hit_node = hit.hit.expect("expected nested child hit");
        let path = ui.debug_node_path(hit_node);
        let child_path = ui.debug_node_path(child_node);
        let child_path_debug = child_path
            .iter()
            .map(|node| {
                (
                    *node,
                    ui.debug_node_bounds(*node),
                    ui.debug_node_clips_hit_test(*node),
                    ui.debug_node_can_scroll_descendant_into_view(*node),
                )
            })
            .collect::<Vec<_>>();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        assert_eq!(
            app.models().get_copied(&child_activated),
            Some(true),
            "expected nested child pressable to activate; hit={hit:?} path={path:?} child={child_node:?} child_path={child_path:?} child_path_debug={child_path_debug:?} child_bounds={child_bounds:?}"
        );

        let selection = app
            .models()
            .read(&state, |st| st.row_selection.clone())
            .ok()
            .unwrap_or_default();
        assert!(
            selection.is_empty(),
            "expected nested child pressable click not to toggle row selection when pointer_row_selection=false"
        );
    }

    #[test]
    fn table_active_descendant_semantics_resolves_from_declarative_active_row_relation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut state_value = TableState::default();
        state_value.pagination.page_size = 3;
        let state = app.models_mut().insert(state_value);

        let data: Arc<[u32]> = Arc::from(vec![0u32, 1u32, 2u32]);
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![{
            let mut col = ColumnDef::new("name");
            col.size = 220.0;
            col
        }]);
        let scroll = VirtualListScrollHandle::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(320.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let props = TableViewProps {
            draw_frame: false,
            row_height: Some(Px(36.0)),
            header_height: Some(Px(40.0)),
            ..Default::default()
        };

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props.clone(),
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        |cx: &mut dyn ElementContextAccess<'_, App>,
                         _col: &ColumnDef<u32>,
                         row: &u32| {
                            crate::ui::text(format!("Row {row}")).into_element(cx.elements())
                        },
                    ),
                    TableDebugIds {
                        row_test_id_prefix: Some(Arc::<str>::from("table-active-desc-row-")),
                        ..Default::default()
                    },
                )]
            })
        };

        let pump =
            |ui: &mut UiTree<App>, app: &mut App, services: &mut FakeServices, frames: usize| {
                for _ in 0..frames {
                    let root = render(ui, app, services);
                    ui.set_root(root);
                    ui.request_semantics_snapshot();
                    ui.layout_all(app, services, bounds, 1.0);
                    let mut scene = fret_core::Scene::default();
                    ui.paint_all(app, services, bounds, &mut scene, 1.0);
                }
            };

        let row_center = |snap: &fret_core::SemanticsSnapshot, row_index: usize| {
            let id = format!("table-active-desc-row-{row_index}");
            let bounds = snap
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(id.as_str()))
                .map(|node| node.bounds)
                .unwrap_or_else(|| panic!("expected row semantics node `{id}`"));
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
            )
        };

        pump(&mut ui, &mut app, &mut services, 2);

        let initial_snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after initial table render");
        let click_pos = row_center(initial_snap, 1);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                is_click: true,
                pointer_id: PointerId(0),
                pointer_type: PointerType::Mouse,
            }),
        );

        // Frame N+1 records the active row element while rebuilding the list body.
        // Frame N+2 lets the parent list semantics resolve the declarative relation against the
        // now-mounted row node for the current frame.
        pump(&mut ui, &mut app, &mut services, 2);

        let snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after activating a row");
        let row = snap
            .nodes
            .iter()
            .find(|node| node.test_id.as_deref() == Some("table-active-desc-row-1"))
            .expect("expected active row semantics node");
        let list = snap
            .nodes
            .iter()
            .find(|node| node.role == SemanticsRole::List)
            .expect("expected table list semantics node after activation");

        assert_eq!(
            list.active_descendant,
            Some(row.id),
            "expected table list semantics to resolve active_descendant to the mounted active row node"
        );
    }

    #[test]
    fn table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut initial = TableState::default();
        initial.pagination.page_size = 8;
        initial.column_pinning.left = vec!["a".into()];
        initial.column_pinning.right = vec!["d".into()];
        let state = app.models_mut().insert(initial);

        let data: Arc<[u32]> = Arc::from((0u32..32).collect::<Vec<_>>());
        let mut col_a = ColumnDef::new("a");
        col_a.size = 120.0;
        let mut col_b = ColumnDef::new("b");
        col_b.size = 280.0;
        let mut col_c = ColumnDef::new("c");
        col_c.size = 240.0;
        let mut col_d = ColumnDef::new("d");
        col_d.size = 140.0;
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![col_a, col_b, col_c, col_d]);

        let scroll = VirtualListScrollHandle::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let props = TableViewProps {
                    overscan: 4,
                    enable_column_grouping: false,
                    ..Default::default()
                };

                let table = table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props,
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        |cx: &mut dyn ElementContextAccess<'_, App>,
                         col: &ColumnDef<u32>,
                         row: &u32| {
                            let cx = cx.elements();
                            cx.text(format!("{}-{row}", col.id.as_ref()))
                        },
                    ),
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "table-retained-colpin-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from("table-retained-colpin-row-")),
                        ..Default::default()
                    },
                );

                vec![cx.semantics(
                    SemanticsProps {
                        test_id: Some(Arc::<str>::from("table-retained-colpin-root")),
                        ..Default::default()
                    },
                    move |_cx| vec![table],
                )]
            })
        };

        let pump = |ui: &mut UiTree<App>,
                    app: &mut App,
                    services: &mut FakeServices,
                    root: &mut fret_core::NodeId| {
            for _ in 0..2 {
                *root = render(ui, app, services);
                ui.set_root(*root);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
                let mut scene = fret_core::Scene::default();
                ui.paint_all(app, services, bounds, &mut scene, 1.0);
            }
        };

        let find_bounds = |snap: &fret_core::SemanticsSnapshot, id: &str| {
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(id))
                .map(|n| n.bounds)
                .unwrap_or_else(|| panic!("expected semantics node `{id}`"))
        };

        let assert_aligned = |snap: &fret_core::SemanticsSnapshot, col: &str| {
            let header_id = format!("table-retained-colpin-header-{col}");
            let row_id = format!("table-retained-colpin-row-0-cell-{col}");
            let header = find_bounds(snap, &header_id);
            let body = find_bounds(snap, &row_id);
            let dx = (header.origin.x.0 - body.origin.x.0).abs();
            let dw = (header.size.width.0 - body.size.width.0).abs();
            assert!(
                dx <= 1.0,
                "expected header/body x alignment for `{col}` (dx={dx:.2})"
            );
            assert!(
                dw <= 1.0,
                "expected header/body width alignment for `{col}` (dw={dw:.2})"
            );
        };

        let mut root = fret_core::NodeId::default();
        pump(&mut ui, &mut app, &mut services, &mut root);

        let mut snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after initial render");
        for col in ["a", "b", "c", "d"] {
            assert_aligned(snap, col);
        }

        let _ = app.models_mut().update(&state, |st| {
            st.column_pinning.left = vec!["a".into()];
            st.column_pinning.right = vec!["d".into()];
            st.column_sizing.insert("b".into(), 320.0);
            st.column_sizing.insert("c".into(), 260.0);
        });
        pump(&mut ui, &mut app, &mut services, &mut root);
        snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after resize update");
        for col in ["a", "b", "c", "d"] {
            assert_aligned(snap, col);
        }

        let root_bounds = find_bounds(snap, "table-retained-colpin-root");
        let center_bounds = find_bounds(snap, "table-retained-colpin-header-c");
        let root_right = root_bounds.origin.x.0 + root_bounds.size.width.0;
        let center_right = center_bounds.origin.x.0 + center_bounds.size.width.0;
        assert!(
            center_right > root_right + 1.0,
            "expected center region overflow to exist (root_right={root_right:.2}, center_right={center_right:.2})"
        );

        let _ = app.models_mut().update(&state, |st| {
            st.column_pinning.left = vec!["a".into()];
            st.column_pinning.right = vec!["c".into(), "d".into()];
        });
        pump(&mut ui, &mut app, &mut services, &mut root);
        snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after pin/unpin update");

        for col in ["a", "b", "c", "d"] {
            assert_aligned(snap, col);
        }
    }

    #[test]
    fn table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let mut initial = TableState::default();
        initial.pagination.page_size = 8;
        initial.column_pinning.left = vec!["a".into()];
        initial.column_pinning.right = vec!["d".into()];
        let state = app.models_mut().insert(initial);

        let data: Arc<[u32]> = Arc::from((0u32..32).collect::<Vec<_>>());
        let mut col_a = ColumnDef::new("a");
        col_a.size = 120.0;
        let mut col_b = ColumnDef::new("b");
        col_b.size = 280.0;
        let mut col_c = ColumnDef::new("c");
        col_c.size = 240.0;
        let mut col_d = ColumnDef::new("d");
        col_d.size = 140.0;
        let columns: Arc<[ColumnDef<u32>]> = Arc::from(vec![col_a, col_b, col_c, col_d]);

        let scroll = VirtualListScrollHandle::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(360.0), Px(220.0)),
        );
        let mut services = FakeServices;

        let render = |ui: &mut UiTree<App>,
                      app: &mut App,
                      services: &mut FakeServices|
         -> fret_core::NodeId {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let props = TableViewProps {
                    overscan: 4,
                    enable_column_grouping: false,
                    row_height: Some(Px(28.0)),
                    row_measure_mode: TableRowMeasureMode::Measured,
                    ..Default::default()
                };

                let table = table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll,
                    0,
                    Arc::new(|_row: &u32, index: usize| RowKey::from_index(index)),
                    None,
                    props,
                    Arc::new(|col: &ColumnDef<u32>| Arc::from(col.id.as_ref())),
                    None,
                    Arc::new(
                        |cx: &mut dyn ElementContextAccess<'_, App>,
                         col: &ColumnDef<u32>,
                         row: &u32| {
                            let cx = cx.elements();
                            if *row == 0 && col.id.as_ref() == "b" {
                                ui::v_stack(|cx| [cx.text("b-0"), cx.text("extra line")])
                                    .gap(Space::N0)
                                    .into_element(cx)
                            } else {
                                cx.text(format!("{}-{row}", col.id.as_ref()))
                            }
                        },
                    ),
                    TableDebugIds {
                        header_cell_test_id_prefix: Some(Arc::<str>::from(
                            "table-retained-colpin-header-",
                        )),
                        row_test_id_prefix: Some(Arc::<str>::from("table-retained-colpin-row-")),
                        ..Default::default()
                    },
                );

                vec![cx.semantics(
                    SemanticsProps {
                        test_id: Some(Arc::<str>::from("table-retained-colpin-root")),
                        ..Default::default()
                    },
                    move |_cx| vec![table],
                )]
            })
        };

        let pump = |ui: &mut UiTree<App>,
                    app: &mut App,
                    services: &mut FakeServices,
                    root: &mut fret_core::NodeId| {
            for _ in 0..2 {
                *root = render(ui, app, services);
                ui.set_root(*root);
                ui.request_semantics_snapshot();
                ui.layout_all(app, services, bounds, 1.0);
                let mut scene = fret_core::Scene::default();
                ui.paint_all(app, services, bounds, &mut scene, 1.0);
            }
        };

        let find_bounds = |snap: &fret_core::SemanticsSnapshot, id: &str| {
            snap.nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some(id))
                .map(|n| n.bounds)
                .unwrap_or_else(|| panic!("expected semantics node `{id}`"))
        };

        let assert_aligned = |snap: &fret_core::SemanticsSnapshot, col: &str| {
            let header_id = format!("table-retained-colpin-header-{col}");
            let row_id = format!("table-retained-colpin-row-0-cell-{col}");
            let header = find_bounds(snap, &header_id);
            let body = find_bounds(snap, &row_id);
            let dx = (header.origin.x.0 - body.origin.x.0).abs();
            let dw = (header.size.width.0 - body.size.width.0).abs();
            assert!(
                dx <= 1.0,
                "expected header/body x alignment for `{col}` (dx={dx:.2})"
            );
            assert!(
                dw <= 1.0,
                "expected header/body width alignment for `{col}` (dw={dw:.2})"
            );
        };

        let mut root = fret_core::NodeId::default();
        pump(&mut ui, &mut app, &mut services, &mut root);

        let snap = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot after initial render");

        for col in ["a", "b", "c", "d"] {
            assert_aligned(snap, col);
        }
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

fn apply_row_pinning_to_paged_rows(
    visible_all: &[DisplayRow],
    page_rows: &[DisplayRow],
    row_pinning: &crate::headless::table::RowPinningState,
) -> Vec<DisplayRow> {
    if row_pinning.top.is_empty() && row_pinning.bottom.is_empty() {
        return page_rows.to_vec();
    }

    let mut pinned: std::collections::HashSet<RowKey> = Default::default();
    pinned.extend(row_pinning.top.iter().copied());
    pinned.extend(row_pinning.bottom.iter().copied());

    let mut by_key: std::collections::HashMap<RowKey, DisplayRow> =
        std::collections::HashMap::new();
    for row in visible_all {
        by_key.entry(row.row_key()).or_insert_with(|| row.clone());
    }

    let mut out: Vec<DisplayRow> =
        Vec::with_capacity(row_pinning.top.len() + page_rows.len() + row_pinning.bottom.len());

    for row_key in &row_pinning.top {
        if let Some(row) = by_key.get(row_key) {
            out.push(row.clone());
        }
    }

    out.extend(
        page_rows
            .iter()
            .filter(|row| !pinned.contains(&row.row_key()))
            .cloned(),
    );

    for row_key in &row_pinning.bottom {
        if let Some(row) = by_key.get(row_key) {
            out.push(row.clone());
        }
    }

    out
}

fn apply_grouped_row_pinning_policy(
    visible_all: &[DisplayRow],
    page_rows_center: &[DisplayRow],
    row_pinning: &crate::headless::table::RowPinningState,
    policy: GroupedRowPinningPolicy,
) -> Vec<DisplayRow> {
    match policy {
        GroupedRowPinningPolicy::PreserveHierarchy => page_rows_center.to_vec(),
        GroupedRowPinningPolicy::PromotePinnedRows => {
            apply_row_pinning_to_paged_rows(visible_all, page_rows_center, row_pinning)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct GroupedBaseDeps {
    items_revision: u64,
    data_len: usize,
    columns_fingerprint: u64,
    grouping: Vec<ColumnId>,
    column_filters: crate::headless::table::ColumnFiltersState,
    global_filter: crate::headless::table::GlobalFilterState,
}

#[derive(Debug, Clone, PartialEq)]
struct GroupedDisplayDeps {
    base: GroupedBaseDeps,
    sorting: crate::headless::table::SortingState,
    expanding: ExpandingState,
    row_pinning: crate::headless::table::RowPinningState,
    grouped_row_pinning_policy: GroupedRowPinningPolicy,
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
    group_aggs_any: std::collections::HashMap<RowKey, Arc<[(ColumnId, TanStackValue)]>>,
    group_aggs_text: GroupAggsText,

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableNavRowMeta {
    row_key: RowKey,
    kind: TableNavRowKind,
    data_index: Option<usize>,
    label: Arc<str>,
}

fn table_collect_leaf_keys(meta: &[TableNavRowMeta]) -> Vec<RowKey> {
    meta.iter()
        .filter(|m| m.kind == TableNavRowKind::Leaf)
        .map(|m| m.row_key)
        .collect()
}

fn table_collect_leaf_keys_in_range(meta: &[TableNavRowMeta], a: usize, b: usize) -> Vec<RowKey> {
    if meta.is_empty() {
        return Vec::new();
    }

    let a = a.min(meta.len().saturating_sub(1));
    let b = b.min(meta.len().saturating_sub(1));
    let (a, b) = if a <= b { (a, b) } else { (b, a) };

    meta.iter()
        .enumerate()
        .filter_map(|(idx, m)| {
            if idx >= a && idx <= b && m.kind == TableNavRowKind::Leaf {
                Some(m.row_key)
            } else {
                None
            }
        })
        .collect()
}

struct TableKeyboardNavState {
    active_index: Rc<Cell<Option<usize>>>,
    anchor_index: Rc<Cell<Option<RowKey>>>,
    row_meta: Rc<RefCell<Arc<[TableNavRowMeta]>>>,
    active_element: Rc<Cell<Option<fret_ui::GlobalElementId>>>,
    active_command: Rc<RefCell<Option<CommandId>>>,
    typeahead: Rc<RefCell<TypeaheadBuffer>>,
    typeahead_timer: Rc<Cell<Option<TimerToken>>>,
}

impl Default for TableKeyboardNavState {
    fn default() -> Self {
        Self {
            active_index: Rc::default(),
            anchor_index: Rc::default(),
            row_meta: Rc::default(),
            active_element: Rc::default(),
            active_command: Rc::default(),
            typeahead: Rc::new(RefCell::new(TypeaheadBuffer::new(0))),
            typeahead_timer: Rc::default(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn table_virtualized<H: UiHost, TData, IHeader, TH, ICell, TC>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: &[ColumnDef<TData>],
    state: impl IntoTableStateModel,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: &RowKeyAt<TData>,
    typeahead_label_at: Option<Arc<TypeaheadLabelAt<TData>>>,
    props: TableViewProps,
    on_row_activate: impl Fn(&Row<'_, TData>) -> Option<CommandId>,
    render_header_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &ColumnDef<TData>,
        Option<bool>,
    ) -> IHeader,
    render_cell: impl FnMut(&mut ElementContext<'_, H>, &Row<'_, TData>, &ColumnDef<TData>) -> ICell,
    output: Option<Model<TableViewOutput>>,
    debug_ids: TableDebugIds,
) -> AnyElement
where
    IHeader: IntoIterator<Item = TH>,
    TH: IntoUiElement<H>,
    ICell: IntoIterator<Item = TC>,
    TC: IntoUiElement<H>,
{
    let state = state.into_table_state_model();
    table_virtualized_impl(
        cx,
        data,
        columns,
        state,
        vertical_scroll,
        items_revision,
        row_key_at,
        typeahead_label_at,
        props,
        None,
        on_row_activate,
        render_header_cell,
        render_cell,
        output,
        debug_ids,
    )
}

/// Virtualized table helper that participates in cross-surface clipboard commands (`edit.copy`).
///
/// `copy_text_at` receives the data index for the selected/active leaf row.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn table_virtualized_copyable<H: UiHost, TData, IHeader, TH, ICell, TC>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: &[ColumnDef<TData>],
    state: impl IntoTableStateModel,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: &RowKeyAt<TData>,
    typeahead_label_at: Option<Arc<TypeaheadLabelAt<TData>>>,
    props: TableViewProps,
    copy_text_at: Arc<CopyTextAtFn>,
    on_row_activate: impl Fn(&Row<'_, TData>) -> Option<CommandId>,
    render_header_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &ColumnDef<TData>,
        Option<bool>,
    ) -> IHeader,
    render_cell: impl FnMut(&mut ElementContext<'_, H>, &Row<'_, TData>, &ColumnDef<TData>) -> ICell,
    output: Option<Model<TableViewOutput>>,
    debug_ids: TableDebugIds,
) -> AnyElement
where
    IHeader: IntoIterator<Item = TH>,
    TH: IntoUiElement<H>,
    ICell: IntoIterator<Item = TC>,
    TC: IntoUiElement<H>,
{
    let state = state.into_table_state_model();
    table_virtualized_impl(
        cx,
        data,
        columns,
        state,
        vertical_scroll,
        items_revision,
        row_key_at,
        typeahead_label_at,
        props,
        Some(copy_text_at),
        on_row_activate,
        render_header_cell,
        render_cell,
        output,
        debug_ids,
    )
}

/// Retained-host virtualized table helper (virt-003 / ADR 0177).
///
/// This is an opt-in surface intended for perf/correctness harnesses. v0 is intentionally minimal:
/// - fixed-height or measured body rows (controlled by `props.row_measure_mode`)
/// - flat (non-grouped) tables only
/// - sorting (including multi-sort state) is supported
/// - focusable "List" semantics with keyboard navigation and typeahead (opt-in labels)
///
/// Typeahead labels:
/// - When `typeahead_label_at` is `Some`, it is used to compute per-row labels for prefix
///   typeahead navigation.
/// - The `usize` argument is the row's `data_index` into `data`.
///
/// The key benefit is that overscan window boundary updates can attach/detach body row subtrees
/// without forcing a parent cache-root rerender under view-cache reuse.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub fn table_virtualized_retained_v0<H: UiHost + 'static, TData>(
    cx: &mut ElementContext<'_, H>,
    data: Arc<[TData]>,
    columns: Arc<[ColumnDef<TData>]>,
    state: impl IntoTableStateModel,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: Arc<RowKeyAt<TData>>,
    typeahead_label_at: Option<Arc<TypeaheadLabelAt<TData>>>,
    props: TableViewProps,
    header_label: Arc<HeaderLabelAt<TData>>,
    header_accessory_at: Option<Arc<HeaderAccessoryAt<H, TData>>>,
    cell_at: Arc<CellAt<H, TData>>,
    debug_ids: TableDebugIds,
) -> AnyElement
where
    TData: 'static,
{
    let state = state.into_table_state_model();
    let TableDebugIds {
        header_row_test_id: debug_header_row_test_id,
        header_cell_test_id_prefix: debug_header_cell_test_id_prefix,
        row_test_id_prefix: debug_row_test_id_prefix,
    } = debug_ids;

    #[derive(Debug, Clone, Copy)]
    struct RowEntry {
        key: RowKey,
        data_index: usize,
    }

    #[derive(Default)]
    struct RetainedTableRowsState {
        last_items_revision: Option<u64>,
        entries: Arc<[RowEntry]>,
    }

    let theme = Theme::global(&*cx.app);
    let (table_bg, border, header_bg, row_hover, row_active) = resolve_table_colors(theme);
    let ring = theme
        .color_by_key("ring")
        .or_else(|| theme.color_by_key("focus.ring"))
        .or_else(|| theme.color_by_key("primary"))
        .unwrap_or(row_active);
    let ring = emphasize_border(ring, 0.9);
    let row_hover_bg = Color {
        a: row_hover.a.min(0.12),
        ..row_hover
    };
    let row_active_bg = Color {
        a: row_active.a.min(0.18),
        ..row_active
    };
    let radius = theme.metric_token("metric.radius.md");

    let row_h = props
        .row_height
        .unwrap_or_else(|| resolve_row_height(theme, props.size));
    let header_h = props.header_height.unwrap_or(row_h);

    let cell_px = resolve_cell_padding_x(theme);
    let cell_py = resolve_cell_padding_y(theme);

    let state_value = cx.watch_model(&state).layout().cloned_or_default();
    let sorting = state_value.sorting.clone();

    let empty: &[TData] = &[];
    let mut sizing_state = state_value.clone();
    if !props.enable_column_grouping {
        sizing_state.grouping.clear();
    }

    let sizing_columns: Vec<ColumnDef<TData>> = columns
        .iter()
        .cloned()
        .map(|c| with_table_view_column_constraints(c, &props))
        .collect();

    let sizing_options = TableOptions {
        grouped_column_mode: props.grouped_column_mode,
        ..Default::default()
    };

    let sizing_table = Table::builder(empty)
        .columns(sizing_columns)
        .state(sizing_state)
        .options(sizing_options)
        .build();
    let core_snapshot = sizing_table.core_model_snapshot();

    let columns: Arc<[ColumnDef<TData>]> = Arc::from(sizing_table.columns().to_vec());

    let visible_columns: Arc<[ColumnDef<TData>]> = Arc::from(
        core_snapshot
            .leaf_columns
            .visible
            .iter()
            .filter_map(|id| sizing_table.column(id.as_ref()).cloned())
            .collect::<Vec<_>>(),
    );

    let col_widths: Arc<[Px]> = Arc::from(
        visible_columns
            .iter()
            .map(|col| {
                let w = core_snapshot
                    .leaf_column_sizing
                    .size
                    .get(&col.id)
                    .copied()
                    .unwrap_or(col.size);
                Px(w)
            })
            .collect::<Vec<_>>(),
    );

    let mut visible_column_index_by_id: std::collections::HashMap<ColumnId, usize> =
        std::collections::HashMap::new();
    for (idx, col) in visible_columns.iter().enumerate() {
        visible_column_index_by_id.insert(col.id.clone(), idx);
    }

    let left_col_indices: Arc<[usize]> = Arc::from(
        core_snapshot
            .leaf_columns
            .left_visible
            .iter()
            .filter_map(|id| visible_column_index_by_id.get(id).copied())
            .collect::<Vec<_>>(),
    );
    let center_col_indices: Arc<[usize]> = Arc::from(
        core_snapshot
            .leaf_columns
            .center_visible
            .iter()
            .filter_map(|id| visible_column_index_by_id.get(id).copied())
            .collect::<Vec<_>>(),
    );
    let right_col_indices: Arc<[usize]> = Arc::from(
        core_snapshot
            .leaf_columns
            .right_visible
            .iter()
            .filter_map(|id| visible_column_index_by_id.get(id).copied())
            .collect::<Vec<_>>(),
    );

    let scroll_x = cx.slot_state(ScrollHandle::default, |h| h.clone());

    let entries = cx.slot_state(RetainedTableRowsState::default, |st| {
        if st.last_items_revision != Some(items_revision) {
            st.last_items_revision = Some(items_revision);

            let mut entries: Vec<RowEntry> = (0..data.len())
                .map(|i| RowEntry {
                    key: (row_key_at)(&data[i], i),
                    data_index: i,
                })
                .collect();

            if !sorting.is_empty() {
                let mut sorters: Vec<SorterSpec<TData>> = Vec::new();
                for spec in &sorting {
                    if let Some(col) = columns
                        .iter()
                        .find(|c| c.id.as_ref() == spec.column.as_ref())
                        && let Some(cmp) = col.sort_cmp.as_ref()
                    {
                        sorters.push((spec.clone(), Arc::clone(cmp)));
                    }
                }

                if !sorters.is_empty() {
                    entries.sort_by(|a, b| {
                        let a_row = &data[a.data_index];
                        let b_row = &data[b.data_index];
                        for (spec, cmp) in &sorters {
                            let ord = (cmp)(a_row, b_row);
                            let ord = if spec.desc { ord.reverse() } else { ord };
                            if ord != std::cmp::Ordering::Equal {
                                return ord;
                            }
                        }

                        a.key.cmp(&b.key)
                    });
                }
            }

            st.entries = Arc::from(entries);
        }

        st.entries.clone()
    });

    let mut fill_layout = LayoutStyle::default();
    fill_layout.size.width = Length::Fill;
    fill_layout.size.height = Length::Fill;
    fill_layout.flex.grow = 1.0;
    fill_layout.flex.basis = Length::Px(Px(0.0));

    let mut options = VirtualListOptions::new(row_h, props.overscan);
    options.items_revision = items_revision;
    options.keep_alive = props
        .keep_alive
        .unwrap_or_else(|| props.overscan.saturating_mul(2));
    match props.row_measure_mode {
        TableRowMeasureMode::Fixed => {
            options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;
            options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;
        }
        TableRowMeasureMode::Measured => {
            options.measure_mode = fret_ui::element::VirtualListMeasureMode::Measured;
            options.key_cache = fret_ui::element::VirtualListKeyCacheMode::AllKeys;
        }
    }

    let header = {
        let state = state.clone();
        let header_label = Arc::clone(&header_label);
        let debug_header_row_test_id = debug_header_row_test_id.clone();
        let debug_header_cell_test_id_prefix = debug_header_cell_test_id_prefix.clone();
        let visible_columns = visible_columns.clone();
        let col_widths = col_widths.clone();
        let left_col_indices = left_col_indices.clone();
        let center_col_indices = center_col_indices.clone();
        let right_col_indices = right_col_indices.clone();
        let scroll_x = scroll_x.clone();
        let sorting = sorting.clone();
        let data_for_header = data.clone();

        let header = cx.container(
            ContainerProps {
                background: Some(header_bg),
                border: Edges {
                    bottom: Px(1.0),
                    ..Default::default()
                },
                border_color: Some(border),
                corner_radii: Corners {
                    top_left: radius,
                    top_right: radius,
                    ..Default::default()
                },
                layout: LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(header_h),
                        min_height: Some(Length::Px(header_h)),
                        max_height: Some(Length::Px(header_h)),
                        ..Default::default()
                    },
                    flex: fret_ui::element::FlexItemStyle {
                        shrink: 0.0,
                        basis: Length::Px(header_h),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx| {
                let render_header_group =
                    |cx: &mut ElementContext<'_, H>,
                     col_indices: Arc<[usize]>,
                     scroll_x_for_group: Option<ScrollHandle>| {
                        let visible_columns = visible_columns.clone();
                        let col_widths = col_widths.clone();
                        let header_label = header_label.clone();
                        let header_accessory_at = header_accessory_at.clone();
                        let debug_header_cell_test_id_prefix =
                            debug_header_cell_test_id_prefix.clone();
                        let sorting = sorting.clone();
                        let state = state.clone();
                        let enable_sorting = props.enable_sorting;
                        let data = data_for_header.clone();

                        let row = ui::h_row(move |cx| {
                            col_indices
                                    .iter()
                                    .map(|col_idx| {
                                        let col = &visible_columns[*col_idx];
                                        let col_w = col_widths[*col_idx];
                                        let label = (header_label)(col);
                                        let header_accessory_at = header_accessory_at.clone();
                                        let sort_state = sort_for_column(&sorting, &col.id);
                                        let col_id = col.id.clone();
                                        let state = state.clone();
                                        let sorting_for_cell = sorting.clone();
                                        let enabled = enable_sorting
                                            && col.enable_sorting
                                            && (col.sort_cmp.is_some() || col.sort_value.is_some());
                                        let sort_options = TableOptions {
                                            enable_sorting,
                                            ..TableOptions::default()
                                        };
                                        let sort_toggle_column = SortToggleColumn {
                                            id: col_id.clone(),
                                            enable_sorting: col.enable_sorting,
                                            enable_multi_sort: col.enable_multi_sort,
                                            sort_desc_first: col.sort_desc_first,
                                            has_sort_value_source: col.sort_cmp.is_some()
                                                || col.sort_value.is_some(),
                                        };
                                        let auto_sort_dir_desc = col
                                            .sort_value
                                            .as_ref()
                                            .and_then(|f| data.first().map(|r| f(r)))
                                            .map(|v| !matches!(v, TanStackValue::String(_)))
                                            .unwrap_or(false);
                                        let debug_test_id: Option<Arc<str>> =
                                            debug_header_cell_test_id_prefix.as_ref().map(
                                                |prefix| {
                                                    Arc::<str>::from(format!(
                                                        "{prefix}{id}",
                                                        id = col_id.as_ref()
                                                    ))
                                                },
                                            );

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
                                                layout: table_fixed_column_fill_layout(col_w),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                vec![cx.pressable(
                                                    PressableProps {
                                                        layout: {
                                                            let mut layout = LayoutStyle::default();
                                                            layout.size.width = Length::Fill;
                                                            layout.size.height = Length::Fill;
                                                            layout
                                                        },
                                                        enabled,
                                                        a11y: PressableA11y {
                                                            role: Some(SemanticsRole::Button),
                                                            label: Some(label.clone()),
                                                            test_id: debug_test_id.clone(),
                                                            ..Default::default()
                                                        },
                                                        ..Default::default()
                                                    },
                                                    move |cx, _st| {
                                                        if enabled {
                                                            let state_model_for_pointer =
                                                                state.clone();
                                                            let sort_toggle_column_for_pointer =
                                                                sort_toggle_column.clone();
                                                            let sort_options_for_pointer =
                                                                sort_options;
                                                            cx.pressable_on_pointer_up(Arc::new(
                                                                move |host, acx, up| {
                                                                    if !up.is_click
                                                                        || up.button
                                                                            != fret_core::MouseButton::Left
                                                                    {
                                                                        return PressablePointerUpResult::Continue;
                                                                    }

                                                                    let multi =
                                                                        up.modifiers.shift;
                                                                    let _ = host.update_model(
                                                                        &state_model_for_pointer,
                                                                        |st| {
                                                                            toggle_sorting_state_handler_tanstack(
                                                                                &mut st.sorting,
                                                                                &sort_toggle_column_for_pointer,
                                                                                sort_options_for_pointer,
                                                                                multi,
                                                                                auto_sort_dir_desc,
                                                                            );
                                                                            st.pagination.page_index = 0;
                                                                        },
                                                                    );
                                                                    host.notify(acx);
                                                                    PressablePointerUpResult::SkipActivate
                                                                },
                                                            ));

                                                            cx.pressable_update_model(
                                                                &state,
                                                                move |st| {
                                                                    toggle_sorting_state_handler_tanstack(
                                                                        &mut st.sorting,
                                                                        &sort_toggle_column,
                                                                        sort_options,
                                                                        false,
                                                                        auto_sort_dir_desc,
                                                                    );
                                                                    st.pagination.page_index = 0;
                                                                },
                                                            );
                                                        }

                                                        let header_text: Arc<str> = match sort_state
                                                        {
                                                            None => label.clone(),
                                                            Some(desc) => {
                                                                let order = if sorting_for_cell
                                                                    .len()
                                                                    > 1
                                                                {
                                                                    sorting_for_cell
                                                                        .iter()
                                                                        .position(|s| {
                                                                            s.column.as_ref()
                                                                                == col_id.as_ref()
                                                                        })
                                                                        .map(|v| v + 1)
                                                                } else {
                                                                    None
                                                                };
                                                                match order {
                                                                    Some(order) => {
                                                                        Arc::<str>::from(format!(
                                                                            "{} {}{}",
                                                                            label,
                                                                            if desc { "▼" } else { "▲" },
                                                                            order
                                                                        ))
                                                                    }
                                                                    None => Arc::<str>::from(format!(
                                                                        "{} {}",
                                                                        label,
                                                                        if desc { "▼" } else { "▲" }
                                                                    )),
                                                                }
                                                            }
                                                        };

                                                        vec![cx.container(
                                                            ContainerProps {
                                                                padding: Edges::symmetric(
                                                                    cell_px, cell_py,
                                                                )
                                                                .into(),
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
                                                            move |_cx| {
                                                                let accessory =
                                                                    header_accessory_at.as_ref().map(|f| f(_cx, col));
                                                                match accessory {
                                                                    None => {
                                                                        vec![_cx.text(header_text.as_ref())]
                                                                    }
                                                                    Some(accessory) => {
                                                                        vec![ui::h_row(move |_cx| {
                                                                            [
                                                                                _cx.text(header_text.as_ref()),
                                                                                _cx.spacer(
                                                                                    SpacerProps::default(),
                                                                                ),
                                                                                accessory,
                                                                            ]
                                                                        })
                                                                        .gap(Space::N1)
                                                                        .justify_start()
                                                                        .items_center()
                                                                        .into_element(_cx)]
                                                                    }
                                                                }
                                                            },
                                                        )]
                                                    },
                                                )]
                                            },
                                        )
                                    })
                                    .collect::<Vec<_>>()
                        })
                        .gap(Space::N0)
                        .justify_start()
                        .items_center()
                        .into_element(cx);

                        table_wrap_horizontal_scroll(cx, scroll_x_for_group, row)
                    };

                vec![ui::h_row(|cx| {
                    let left = render_header_group(cx, left_col_indices.clone(), None);
                    let center =
                        render_header_group(cx, center_col_indices.clone(), Some(scroll_x.clone()));
                    let right = render_header_group(cx, right_col_indices.clone(), None);
                    [left, center, right]
                })
                .gap(Space::N0)
                .justify_start()
                .items_stretch()
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx)]
            },
        );

        if let Some(test_id) = debug_header_row_test_id {
            header.test_id(test_id)
        } else {
            header
        }
    };

    let key_at: Arc<dyn Fn(usize) -> fret_ui::ItemKey> = {
        let entries = entries.clone();
        Arc::new(move |i| entries[i].key.0)
    };

    struct RetainedTableKeyboardNavState {
        active_index: Rc<Cell<Option<usize>>>,
        anchor_index: Rc<Cell<Option<RowKey>>>,
        active_element: Rc<Cell<Option<fret_ui::GlobalElementId>>>,
        labels: Rc<RefCell<Arc<[Arc<str>]>>>,
        disabled: Rc<RefCell<Arc<[bool]>>>,
        last_labels_revision: Cell<Option<u64>>,
        typeahead: Rc<RefCell<TypeaheadBuffer>>,
        typeahead_timer: Rc<Cell<Option<TimerToken>>>,
    }

    impl Default for RetainedTableKeyboardNavState {
        fn default() -> Self {
            Self {
                active_index: Rc::default(),
                anchor_index: Rc::default(),
                active_element: Rc::default(),
                labels: Rc::new(RefCell::new(Arc::from([]))),
                disabled: Rc::new(RefCell::new(Arc::from([]))),
                last_labels_revision: Cell::new(None),
                typeahead: Rc::new(RefCell::new(TypeaheadBuffer::new(0))),
                typeahead_timer: Rc::default(),
            }
        }
    }

    let (
        active_index,
        anchor_index,
        active_element,
        labels,
        disabled,
        _last_labels_revision,
        typeahead,
        typeahead_timer,
    ) = cx.slot_state(RetainedTableKeyboardNavState::default, |nav| {
        if nav.last_labels_revision.get() != Some(items_revision) {
            nav.last_labels_revision.set(Some(items_revision));

            if let Some(typeahead_label_at) = &typeahead_label_at {
                let mut next_labels: Vec<Arc<str>> = Vec::with_capacity(entries.len());
                let mut next_disabled: Vec<bool> = Vec::with_capacity(entries.len());
                for entry in entries.iter() {
                    let i = entry.data_index;
                    let label = (typeahead_label_at)(&data[i], i);
                    let disabled = label.trim().is_empty();
                    next_labels.push(label);
                    next_disabled.push(disabled);
                }
                *nav.labels.borrow_mut() = Arc::from(next_labels);
                *nav.disabled.borrow_mut() = Arc::from(next_disabled);
            } else {
                let mut next_labels: Vec<Arc<str>> = Vec::with_capacity(entries.len());
                let mut next_disabled: Vec<bool> = Vec::with_capacity(entries.len());
                for entry in entries.iter() {
                    let label: Arc<str> = Arc::from(entry.key.0.to_string());
                    next_labels.push(label);
                    next_disabled.push(false);
                }
                *nav.labels.borrow_mut() = Arc::from(next_labels);
                *nav.disabled.borrow_mut() = Arc::from(next_disabled);
            }
        }

        (
            nav.active_index.clone(),
            nav.anchor_index.clone(),
            nav.active_element.clone(),
            nav.labels.clone(),
            nav.disabled.clone(),
            nav.last_labels_revision.clone(),
            nav.typeahead.clone(),
            nav.typeahead_timer.clone(),
        )
    });

    let row_builder =
        {
            let state = state.clone();
            let entries = entries.clone();
            let data = data.clone();
            let props = props.clone();
            let visible_columns = visible_columns.clone();
            let col_widths = col_widths.clone();
            let cell_at = Arc::clone(&cell_at);
            let debug_row_test_id_prefix = debug_row_test_id_prefix.clone();
            let left_col_indices = left_col_indices.clone();
            let center_col_indices = center_col_indices.clone();
            let right_col_indices = right_col_indices.clone();
            let scroll_x = scroll_x.clone();
            let active_index_for_row_builder = active_index.clone();
            let active_element_for_row_builder = active_element.clone();
            let anchor_index_for_row_builder = anchor_index.clone();

            move |key_handler: fret_ui::action::OnKeyDown, focus_target: GlobalElementId| {
                let active_index_for_row = active_index_for_row_builder.clone();
                let active_element_for_row = active_element_for_row_builder.clone();
                let anchor_index_for_row = anchor_index_for_row_builder.clone();
                Arc::new(move |cx: &mut ElementContext<'_, H>, i: usize| {
                    let entry = entries[i];
                    let row_key = entry.key;
                    let data_index = entry.data_index;

                    let selected = cx
                        .watch_model(&state)
                        .paint()
                        .read_ref(|s| s.row_selection.contains(&row_key))
                        .ok()
                        .unwrap_or(false);

                    let test_id = debug_row_test_id_prefix
                        .as_ref()
                        .map(|prefix| Arc::<str>::from(format!("{}{id}", prefix, id = row_key.0)));

                    let state_model = state.clone();
                    let entries = entries.clone();
                    let anchor_index = anchor_index_for_row.clone();
                    let data_for_row = Arc::clone(&data);
                    let columns_for_row = Arc::clone(&visible_columns);
                    let col_widths_for_row = col_widths.clone();
                    let cell_at_for_row = Arc::clone(&cell_at);
                    let key_handler_for_row = key_handler.clone();
                    let focus_target_for_row = focus_target;
                    let row_cell_test_id_prefix = debug_row_test_id_prefix.clone();
                    let left_col_indices_for_row = left_col_indices.clone();
                    let center_col_indices_for_row = center_col_indices.clone();
                    let right_col_indices_for_row = right_col_indices.clone();
                    let scroll_x_for_row = scroll_x.clone();
                    let active_index = active_index_for_row.clone();
                    let active_element = active_element_for_row.clone();
                    let focus_target = focus_target_for_row;
                    let single = props.single_row_selection;
                    let policy = props.pointer_row_selection_policy;
                    let pointer_row_selection_enabled =
                        props.enable_row_selection && props.pointer_row_selection;
                    let row_wrapper_layout = retained_table_row_fill_layout();

                    let render_row_visuals =
                        |cx: &mut ElementContext<'_, H>, hovered: bool, pressed: bool| {
                            let bg = if selected || pressed {
                                Some(row_active_bg)
                            } else if hovered {
                                Some(row_hover_bg)
                            } else {
                                None
                            };

                            vec![retained_table_render_row_visuals(
                                cx,
                                data_for_row.clone(),
                                data_index,
                                row_key,
                                bg,
                                props.clone(),
                                border,
                                cell_px,
                                cell_py,
                                key_handler_for_row.clone(),
                                columns_for_row.clone(),
                                col_widths_for_row.clone(),
                                cell_at_for_row.clone(),
                                row_cell_test_id_prefix.clone(),
                                left_col_indices_for_row.clone(),
                                center_col_indices_for_row.clone(),
                                right_col_indices_for_row.clone(),
                                scroll_x_for_row.clone(),
                            )]
                        };

                    if pointer_row_selection_enabled {
                        cx.pressable_with_id(
                            PressableProps {
                                enabled: true,
                                focusable: false,
                                a11y: PressableA11y {
                                    role: Some(SemanticsRole::ListItem),
                                    test_id,
                                    selected,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, st, id| {
                                let active_index_for_pointer_down = active_index.clone();
                                cx.pressable_add_on_pointer_down(Arc::new(
                                    move |_host, action_cx, down| {
                                        if down.button != fret_core::MouseButton::Left {
                                            return PressablePointerDownResult::Continue;
                                        }
                                        if down
                                            .hit_pressable_target
                                            .is_some_and(|t| t != action_cx.target)
                                        {
                                            return PressablePointerDownResult::Continue;
                                        }
                                        active_index_for_pointer_down.set(Some(i));
                                        PressablePointerDownResult::Continue
                                    },
                                ));
                                if policy == PointerRowSelectionPolicy::ListLike {
                                    let anchor_index_for_pointer_down = anchor_index.clone();
                                    let row_key_for_anchor = row_key;
                                    cx.pressable_add_on_pointer_down(Arc::new(
                                        move |_host, action_cx, down| {
                                            if down.button != fret_core::MouseButton::Left {
                                                return PressablePointerDownResult::Continue;
                                            }
                                            if down
                                                .hit_pressable_target
                                                .is_some_and(|t| t != action_cx.target)
                                            {
                                                return PressablePointerDownResult::Continue;
                                            }
                                            let next_anchor = if down.modifiers.shift {
                                                anchor_index_for_pointer_down
                                                    .get()
                                                    .or(Some(row_key_for_anchor))
                                            } else {
                                                Some(row_key_for_anchor)
                                            };
                                            anchor_index_for_pointer_down.set(next_anchor);
                                            PressablePointerDownResult::Continue
                                        },
                                    ));
                                }

                                cx.pressable_on_pointer_up(Arc::new(move |host, action_cx, up| {
                                    if up.button != fret_core::MouseButton::Left || !up.is_click {
                                        return PressablePointerUpResult::Continue;
                                    }
                                    if up
                                        .down_hit_pressable_target
                                        .is_some_and(|t| t != action_cx.target)
                                    {
                                        return PressablePointerUpResult::Continue;
                                    }
                                    host.request_focus(focus_target);
                                    let modifiers = up.modifiers;
                                    let mut range_keys: Option<Vec<RowKey>> = None;
                                    if policy == PointerRowSelectionPolicy::ListLike
                                        && !single
                                        && modifiers.shift
                                    {
                                        let anchor_key = anchor_index.get().unwrap_or(row_key);
                                        let anchor = entries
                                            .iter()
                                            .position(|entry| entry.key == anchor_key)
                                            .unwrap_or(i);
                                        let (a, b) = if anchor <= i {
                                            (anchor, i)
                                        } else {
                                            (i, anchor)
                                        };
                                        let keys = entries
                                            .iter()
                                            .enumerate()
                                            .filter_map(|(idx, entry)| {
                                                (idx >= a && idx <= b).then_some(entry.key)
                                            })
                                            .collect::<Vec<_>>();
                                        if !keys.is_empty() {
                                            range_keys = Some(keys);
                                        }
                                    }

                                    let anchor_index_for_update = anchor_index.clone();
                                    let _ = host.models_mut().update(&state_model, move |st| {
                                        match policy {
                                            PointerRowSelectionPolicy::Toggle => {
                                                let selected = st.row_selection.contains(&row_key);
                                                if single {
                                                    st.row_selection.clear();
                                                }
                                                if selected {
                                                    st.row_selection.remove(&row_key);
                                                } else {
                                                    st.row_selection.insert(row_key);
                                                }
                                            }
                                            PointerRowSelectionPolicy::ListLike => {
                                                if let Some(range_keys) = range_keys.clone() {
                                                    if modifiers.ctrl || modifiers.meta {
                                                        st.row_selection
                                                            .extend(range_keys.iter().copied());
                                                    } else {
                                                        st.row_selection.clear();
                                                        st.row_selection
                                                            .extend(range_keys.iter().copied());
                                                    }
                                                } else if !single
                                                    && (modifiers.ctrl || modifiers.meta)
                                                {
                                                    if st.row_selection.contains(&row_key) {
                                                        st.row_selection.remove(&row_key);
                                                    } else {
                                                        st.row_selection.insert(row_key);
                                                    }
                                                } else {
                                                    st.row_selection.clear();
                                                    st.row_selection.insert(row_key);
                                                }
                                            }
                                        }
                                    });
                                    if policy == PointerRowSelectionPolicy::ListLike {
                                        let next_anchor = if modifiers.shift {
                                            anchor_index_for_update.get().or(Some(row_key))
                                        } else {
                                            Some(row_key)
                                        };
                                        anchor_index_for_update.set(next_anchor);
                                    } else {
                                        anchor_index_for_update.set(Some(row_key));
                                    }
                                    host.request_redraw(action_cx.window);
                                    PressablePointerUpResult::SkipActivate
                                }));

                                if active_index.get() == Some(i) {
                                    active_element.set(Some(id));
                                }
                                render_row_visuals(cx, st.hovered, st.pressed)
                            },
                        )
                    } else {
                        cx.semantics_with_id(
                            SemanticsProps {
                                layout: row_wrapper_layout,
                                role: SemanticsRole::ListItem,
                                test_id,
                                selected,
                                ..Default::default()
                            },
                            move |cx, id| {
                                if active_index.get() == Some(i) {
                                    active_element.set(Some(id));
                                }
                                vec![cx.hover_region(
                                    HoverRegionProps {
                                        layout: row_wrapper_layout,
                                    },
                                    move |cx, hovered| render_row_visuals(cx, hovered, false),
                                )]
                            },
                        )
                    }
                })
            }
        };

    let list = cx.semantics_with_id(
        SemanticsProps {
            role: SemanticsRole::List,
            focusable: true,
            ..Default::default()
        },
        move |cx, list_id| {
            let state_for_keys = state.clone();
            let vertical_scroll_for_keys = vertical_scroll.clone();
            let entries_for_keys = entries.clone();
            let entries_for_list = entries.clone();
            let labels_for_keys = labels.clone();
            let disabled_for_keys = disabled.clone();
            let active_index_for_keys = active_index.clone();
            let anchor_index_for_keys = anchor_index.clone();
            let typeahead_for_keys = typeahead.clone();
            let typeahead_timer_for_keys = typeahead_timer.clone();

            let key_handler: fret_ui::action::OnKeyDown = Arc::new(move |host, action_cx, down| {
                let Some(len) = entries_for_keys.len().checked_sub(1) else {
                    return false;
                };

                let current = active_index_for_keys.get().unwrap_or(0).min(len);

                let cancel_typeahead_timer =
                    |host: &mut dyn fret_ui::action::UiActionHost,
                     typeahead_timer: &Rc<Cell<Option<TimerToken>>>| {
                        if let Some(token) = typeahead_timer.get() {
                            host.push_effect(Effect::CancelTimer { token });
                            typeahead_timer.set(None);
                        }
                    };

                match down.key {
                    KeyCode::ArrowDown => {
                        let next = (current + 1).min(len);
                        active_index_for_keys.set(Some(next));
                        anchor_index_for_keys.set(Some(entries_for_keys[next].key));
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        vertical_scroll_for_keys.scroll_to_item(next, ScrollStrategy::Nearest);
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::ArrowUp => {
                        let next = current.saturating_sub(1);
                        active_index_for_keys.set(Some(next));
                        anchor_index_for_keys.set(Some(entries_for_keys[next].key));
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        vertical_scroll_for_keys.scroll_to_item(next, ScrollStrategy::Nearest);
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::Home => {
                        active_index_for_keys.set(Some(0));
                        anchor_index_for_keys.set(Some(entries_for_keys[0].key));
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        vertical_scroll_for_keys.scroll_to_item(0, ScrollStrategy::Nearest);
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::End => {
                        active_index_for_keys.set(Some(len));
                        anchor_index_for_keys.set(Some(entries_for_keys[len].key));
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        vertical_scroll_for_keys.scroll_to_item(len, ScrollStrategy::Nearest);
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::Escape => {
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        host.request_redraw(action_cx.window);
                        true
                    }
                    KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                        if !props.enable_row_selection {
                            return false;
                        }
                        let row_key = entries_for_keys[current].key;
                        let _ = host.models_mut().update(&state_for_keys, move |st| {
                            let selected = st.row_selection.contains(&row_key);
                            if props.single_row_selection {
                                st.row_selection.clear();
                            }
                            if selected {
                                st.row_selection.remove(&row_key);
                            } else {
                                st.row_selection.insert(row_key);
                            }
                        });
                        anchor_index_for_keys.set(Some(row_key));
                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        typeahead_for_keys.borrow_mut().clear();
                        host.request_redraw(action_cx.window);
                        true
                    }
                    _ => {
                        if down.repeat {
                            return false;
                        }
                        let Some(input) = fret_core::keycode_to_ascii_lowercase(down.key) else {
                            return false;
                        };

                        typeahead_for_keys.borrow_mut().push_char(input, 0);
                        let typeahead_buf = typeahead_for_keys.borrow();
                        let Some(query) = typeahead_buf.query(0) else {
                            return false;
                        };

                        let labels = labels_for_keys.borrow().clone();
                        let disabled = disabled_for_keys.borrow().clone();
                        let next =
                            match_prefix_arc_str(&labels, &disabled, query, Some(current), true);
                        if let Some(next) = next
                            && next != current
                        {
                            active_index_for_keys.set(Some(next));
                            anchor_index_for_keys.set(Some(entries_for_keys[next].key));
                            // Typeahead should ensure the matched row becomes *visibly* in-view,
                            // not just "present in overscan".
                            vertical_scroll_for_keys.scroll_to_item(next, ScrollStrategy::Start);
                        }

                        cancel_typeahead_timer(host, &typeahead_timer_for_keys);
                        let token = host.next_timer_token();
                        typeahead_timer_for_keys.set(Some(token));
                        host.push_effect(Effect::SetTimer {
                            window: Some(action_cx.window),
                            token,
                            after: TABLE_TYPEAHEAD_TIMEOUT,
                            repeat: None,
                        });

                        host.request_redraw(action_cx.window);
                        true
                    }
                }
            });

            cx.key_on_key_down_for(list_id, key_handler.clone());
            let row = row_builder(key_handler, list_id);

            {
                let typeahead = typeahead.clone();
                let typeahead_timer = typeahead_timer.clone();
                cx.timer_on_timer_for(
                    list_id,
                    Arc::new(move |host, action_cx, token| {
                        if typeahead_timer.get() != Some(token) {
                            return false;
                        }
                        typeahead_timer.set(None);
                        typeahead.borrow_mut().clear();
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
            }

            let content =
                cx.pointer_region(fret_ui::element::PointerRegionProps::default(), move |cx| {
                    let focus_id = list_id;
                    cx.pointer_region_on_pointer_up(Arc::new(move |host, action_cx, up| {
                        if up.button != fret_core::MouseButton::Left || !up.is_click {
                            return false;
                        }
                        if up.down_hit_pressable_target.is_some() {
                            return false;
                        }
                        host.request_focus(focus_id);
                        host.request_redraw(action_cx.window);
                        false
                    }));

                    vec![
                        ui::v_flex(move |cx| {
                            [
                                header,
                                cx.virtual_list_keyed_retained_with_layout(
                                    fill_layout,
                                    entries_for_list.len(),
                                    options,
                                    vertical_scroll,
                                    key_at,
                                    row,
                                ),
                            ]
                        })
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N0)
                        .justify_start()
                        .items_stretch()
                        .into_element(cx),
                    ]
                });

            let focus_ring = RingStyle {
                placement: RingPlacement::Inset,
                width: Px(2.0),
                offset: Px(0.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(radius),
            };

            vec![cx.container(
                ContainerProps {
                    layout: fill_layout,
                    background: Some(table_bg),
                    border: if props.draw_frame {
                        Edges::all(Px(1.0))
                    } else {
                        Edges::all(Px(0.0))
                    },
                    border_color: if props.draw_frame { Some(border) } else { None },
                    focus_border_color: if props.draw_frame { Some(ring) } else { None },
                    focus_ring: Some(focus_ring),
                    focus_within: true,
                    corner_radii: if props.draw_frame {
                        Corners::all(radius)
                    } else {
                        Corners::all(Px(0.0))
                    },
                    ..Default::default()
                },
                move |_cx| vec![content],
            )]
        },
    );

    if let Some(active_element) = active_element.get() {
        list.attach_semantics(
            SemanticsDecoration::default().active_descendant_element(active_element.0),
        )
    } else {
        list
    }
}

#[allow(clippy::too_many_arguments)]
#[track_caller]
fn table_virtualized_impl<H: UiHost, TData, IHeader, TH, ICell, TC>(
    cx: &mut ElementContext<'_, H>,
    data: &[TData],
    columns: &[ColumnDef<TData>],
    state: Model<TableState>,
    vertical_scroll: &VirtualListScrollHandle,
    items_revision: u64,
    row_key_at: &RowKeyAt<TData>,
    typeahead_label_at: Option<Arc<TypeaheadLabelAt<TData>>>,
    props: TableViewProps,
    copy_text_at: Option<Arc<CopyTextAtFn>>,
    on_row_activate: impl Fn(&Row<'_, TData>) -> Option<CommandId>,
    mut render_header_cell: impl FnMut(
        &mut ElementContext<'_, H>,
        &ColumnDef<TData>,
        Option<bool>,
    ) -> IHeader,
    mut render_cell: impl FnMut(&mut ElementContext<'_, H>, &Row<'_, TData>, &ColumnDef<TData>) -> ICell,
    output: Option<Model<TableViewOutput>>,
    debug_ids: TableDebugIds,
) -> AnyElement
where
    IHeader: IntoIterator<Item = TH>,
    TH: IntoUiElement<H>,
    ICell: IntoIterator<Item = TC>,
    TC: IntoUiElement<H>,
{
    let profile = std::env::var_os("FRET_TABLE_PROFILE").is_some();
    let TableDebugIds {
        header_row_test_id: debug_header_row_test_id,
        header_cell_test_id_prefix: debug_header_cell_test_id_prefix,
        row_test_id_prefix: debug_row_test_id_prefix,
    } = debug_ids;
    let state_value = cx.watch_model(&state).layout().cloned_or_default();

    let theme = Theme::global(&*cx.app);
    let (table_bg, border, header_bg, row_hover, row_active) = resolve_table_colors(theme);
    let resize_grip = emphasize_border(border, 0.35);
    let resize_preview = emphasize_border(border, 0.75);
    let ring = theme
        .color_by_key("ring")
        .or_else(|| theme.color_by_key("focus.ring"))
        .or_else(|| theme.color_by_key("primary"))
        .unwrap_or(row_active);
    let ring = emphasize_border(ring, 0.9);
    let row_hover_bg = Color {
        a: row_hover.a.min(0.12),
        ..row_hover
    };
    let row_active_bg = Color {
        a: row_active.a.min(0.18),
        ..row_active
    };
    let radius = theme.metric_token("metric.radius.md");

    let row_h = props
        .row_height
        .unwrap_or_else(|| resolve_row_height(theme, props.size));
    let header_h = props.header_height.unwrap_or(row_h);
    let body_row_height = match props.row_measure_mode {
        TableRowMeasureMode::Fixed => Length::Px(row_h),
        TableRowMeasureMode::Measured => Length::Auto,
    };
    let cell_px = resolve_cell_padding_x(theme);
    let cell_py = resolve_cell_padding_y(theme);

    let scroll_x = cx.slot_state(ScrollHandle::default, |h| h.clone());

    let grouping = if props.enable_column_grouping {
        state_value.grouping.as_slice()
    } else {
        &[]
    };

    let empty: &[TData] = &[];
    let mut sizing_state = state_value.clone();
    if !props.enable_column_grouping {
        sizing_state.grouping.clear();
    }

    let sizing_columns: Vec<ColumnDef<TData>> = columns
        .iter()
        .cloned()
        .map(|c| with_table_view_column_constraints(c, &props))
        .collect();
    let sizing_options = TableOptions {
        grouped_column_mode: props.grouped_column_mode,
        ..Default::default()
    };

    let sizing_table = Table::builder(empty)
        .columns(sizing_columns)
        .state(sizing_state)
        .options(sizing_options)
        .build();
    let core_snapshot = sizing_table.core_model_snapshot();

    let mut column_width_by_id: std::collections::HashMap<ColumnId, Px> =
        std::collections::HashMap::new();
    for col in columns {
        let w = core_snapshot
            .leaf_column_sizing
            .size
            .get(&col.id)
            .copied()
            .unwrap_or(col.size);
        column_width_by_id.insert(col.id.clone(), Px(w));
    }
    let column_width_by_id: Arc<std::collections::HashMap<ColumnId, Px>> =
        Arc::new(column_width_by_id);

    let col_by_id_for_layout: std::collections::HashMap<&str, &ColumnDef<TData>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    let visible_columns: Vec<&ColumnDef<TData>> = core_snapshot
        .leaf_columns
        .visible
        .iter()
        .filter_map(|id| col_by_id_for_layout.get(id.as_ref()).copied())
        .collect();
    let left_cols: Vec<&ColumnDef<TData>> = core_snapshot
        .leaf_columns
        .left_visible
        .iter()
        .filter_map(|id| col_by_id_for_layout.get(id.as_ref()).copied())
        .collect();
    let center_cols: Vec<&ColumnDef<TData>> = core_snapshot
        .leaf_columns
        .center_visible
        .iter()
        .filter_map(|id| col_by_id_for_layout.get(id.as_ref()).copied())
        .collect();
    let right_cols: Vec<&ColumnDef<TData>> = core_snapshot
        .leaf_columns
        .right_visible
        .iter()
        .filter_map(|id| col_by_id_for_layout.get(id.as_ref()).copied())
        .collect();

    let sorting_key = if grouping.is_empty() {
        state_value.sorting.clone()
    } else {
        Vec::new()
    };

    let has_row_pinning =
        grouping.is_empty() && is_some_rows_pinned(&state_value.row_pinning, None);
    let row_order = if grouping.is_empty() && !has_row_pinning {
        Some(cx.slot_state(FlatRowOrderCache::default, |cache| {
            let deps = FlatRowOrderDeps {
                items_revision,
                data_len: data.len(),
                sorting: sorting_key.clone(),
                column_filters: state_value.column_filters.clone(),
                global_filter: state_value.global_filter.clone(),
            };

            let started = Instant::now();
            let (order, recomputed) = cache.row_order(data, columns, deps);
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
        }))
    } else {
        None
    };

    let page_display_rows: Vec<DisplayRow> = if grouping.is_empty() {
        if has_row_pinning {
            let options = crate::headless::table::TableOptions {
                keep_pinned_rows: props.keep_pinned_rows,
                ..Default::default()
            };

            let table_pre = Table::builder(data)
                .columns(columns.to_vec())
                .global_filter_fn(FilteringFnSpec::Auto)
                .get_row_key(|row, idx, _parent| row_key_at(row, idx))
                .state(state_value.clone())
                .options(options)
                .build();

            let total_rows = table_pre.pre_pagination_row_model().root_rows().len();
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

            let mut render_state = state_value.clone();
            render_state.pagination.page_index = bounds.page_index;
            let table = Table::builder(data)
                .columns(columns.to_vec())
                .global_filter_fn(FilteringFnSpec::Auto)
                .get_row_key(|row, idx, _parent| row_key_at(row, idx))
                .state(render_state)
                .options(options)
                .build();

            let core = table.core_row_model();
            table
                .top_row_keys()
                .into_iter()
                .chain(table.center_row_keys())
                .chain(table.bottom_row_keys())
                .filter_map(|row_key| {
                    let row_index = core.row_by_key(row_key)?;
                    let row = core.row(row_index)?;
                    if row.parent.is_some() {
                        return None;
                    }
                    let data_index = row.index;
                    if data_index >= data.len() {
                        return None;
                    }
                    Some(DisplayRow::Leaf {
                        data_index,
                        row_key,
                        depth: row.depth as usize,
                    })
                })
                .collect()
        } else {
            let row_order = row_order.expect("row_order");
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
        }
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
            row_pinning: state_value.row_pinning.clone(),
            grouped_row_pinning_policy: props.grouped_row_pinning_policy,
            page_index: state_value.pagination.page_index,
            page_size: state_value.pagination.page_size,
        };

        let (page_rows, view_output, clamp_to_page): (
            Vec<DisplayRow>,
            TableViewOutput,
            Option<usize>,
        ) = cx.slot_state(GroupedDisplayCache::default, |cache| {
            if cache.deps.as_ref() == Some(&deps) {
                return (cache.page_rows.clone(), cache.output.clone(), None);
            }

            if cache.base_deps.as_ref() == Some(&deps.base) {
                let grouped = &cache.grouped;
                let row_index_by_key = &cache.row_index_by_key;
                let group_labels = &cache.group_labels;
                let group_aggs_text = &cache.group_aggs_text;
                let group_aggs_u64 = &cache.group_aggs_u64;
                let group_aggs_any = &cache.group_aggs_any;

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
                    group_aggs_any,
                );

                for root in roots {
                    push_visible(
                        grouped,
                        root,
                        row_index_by_key,
                        group_labels,
                        group_aggs_text,
                        group_aggs_u64,
                        group_aggs_any,
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

                let page_rows_center: Vec<DisplayRow> = if bounds.page_count == 0 {
                    Vec::new()
                } else {
                    visible
                        .get(bounds.page_start..bounds.page_end)
                        .unwrap_or_default()
                        .to_vec()
                };

                let page_rows = apply_grouped_row_pinning_policy(
                    &visible,
                    &page_rows_center,
                    &deps.row_pinning,
                    deps.grouped_row_pinning_policy,
                );

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

            let options = crate::headless::table::TableOptions {
                manual_sorting: true,
                manual_pagination: true,
                manual_expanding: true,
                keep_pinned_rows: props.keep_pinned_rows,
                ..Default::default()
            };

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
            ) -> (GroupAggsU64, GroupAggsText) {
                if agg_columns.is_empty() {
                    return (Default::default(), Default::default());
                }
                let out_u64 =
                    compute_grouped_u64_aggregations(model, data, row_index_by_key, agg_columns);

                let mut out_text: GroupAggsText = Default::default();
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

            fn push_visible<TData>(
                model: &crate::headless::table::GroupedRowModel,
                index: crate::headless::table::GroupedRowIndex,
                row_index_by_key: &std::collections::HashMap<RowKey, usize>,
                group_labels: &std::collections::HashMap<RowKey, Arc<str>>,
                group_aggs_text: &GroupAggsText,
                group_aggs_u64: &GroupAggsU64,
                group_aggs_any: &GroupAggsAny,
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

                            if !sorting.is_empty() {
                                let mut owned = row.sub_rows.clone();
                                sort_grouped_row_indices_in_place(
                                    model,
                                    &mut owned,
                                    sorting,
                                    columns,
                                    data,
                                    row_index_by_key,
                                    group_aggs_u64,
                                    group_aggs_any,
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
                                    group_aggs_any,
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
            let group_aggs_any = table.grouped_aggregations_any().clone();

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
                &group_aggs_any,
            );

            for root in roots {
                push_visible(
                    &grouped,
                    root,
                    &row_index_by_key,
                    &group_labels,
                    &group_aggs_text,
                    &group_aggs_u64,
                    &group_aggs_any,
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

            let page_rows_center: Vec<DisplayRow> = if bounds.page_count == 0 {
                Vec::new()
            } else {
                visible
                    .get(bounds.page_start..bounds.page_end)
                    .unwrap_or_default()
                    .to_vec()
            };

            let page_rows = apply_grouped_row_pinning_policy(
                &visible,
                &page_rows_center,
                &deps.row_pinning,
                deps.grouped_row_pinning_policy,
            );

            cache.base_deps = Some(deps.base.clone());
            cache.grouped = grouped;
            cache.row_index_by_key = row_index_by_key;
            cache.group_labels = group_labels;
            cache.group_aggs_u64 = group_aggs_u64;
            cache.group_aggs_any = group_aggs_any;
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
    list_options.keep_alive = props
        .keep_alive
        .unwrap_or_else(|| props.overscan.saturating_mul(2));
    match props.row_measure_mode {
        TableRowMeasureMode::Fixed => {
            list_options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;
            list_options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;
        }
        TableRowMeasureMode::Measured => {
            list_options.measure_mode = fret_ui::element::VirtualListMeasureMode::Measured;
            list_options.key_cache = fret_ui::element::VirtualListKeyCacheMode::AllKeys;
        }
    }

    let rendered_rows = Cell::new(0usize);

    let (
        active_index,
        anchor_index,
        row_meta,
        active_element,
        active_command,
        typeahead,
        typeahead_timer,
    ) = cx.slot_state(TableKeyboardNavState::default, |st| {
        (
            st.active_index.clone(),
            st.anchor_index.clone(),
            st.row_meta.clone(),
            st.active_element.clone(),
            st.active_command.clone(),
            st.typeahead.clone(),
            st.typeahead_timer.clone(),
        )
    });

    {
        let typeahead_label_at = typeahead_label_at.clone();
        let typeahead_facet_str_fn = visible_columns.iter().find_map(|c| c.facet_str_fn.clone());
        let next_meta: Arc<[TableNavRowMeta]> = page_display_rows
            .iter()
            .map(|row| match row {
                DisplayRow::Leaf {
                    data_index,
                    row_key,
                    ..
                } => {
                    let label = typeahead_label_at
                        .as_ref()
                        .map(|f| f(&data[*data_index], *data_index))
                        .or_else(|| {
                            typeahead_facet_str_fn
                                .as_ref()
                                .map(|f| Arc::<str>::from(f(&data[*data_index]).to_string()))
                        })
                        .unwrap_or_else(|| Arc::from(""));
                    TableNavRowMeta {
                        row_key: *row_key,
                        kind: TableNavRowKind::Leaf,
                        data_index: Some(*data_index),
                        label,
                    }
                }
                DisplayRow::Group { row_key, label, .. } => TableNavRowMeta {
                    row_key: *row_key,
                    kind: TableNavRowKind::Group,
                    data_index: None,
                    label: label.clone(),
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
        let anchor_index = anchor_index.clone();
        let row_meta = row_meta.clone();
        let active_command = active_command.clone();
        let vertical_scroll = vertical_scroll.clone();
        let state = state.clone();
        let enable_row_selection = props.enable_row_selection;
        let single_row_selection = props.single_row_selection;
        let typeahead = typeahead.clone();
        let typeahead_timer = typeahead_timer.clone();

        Arc::new(move |host, action_cx, down| {
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
            let has_disallowed_mods = down.modifiers.alt || down.modifiers.alt_gr;
            if has_disallowed_mods {
                return false;
            }
            let primary = (down.modifiers.ctrl || down.modifiers.meta) && !down.modifiers.alt_gr;

            let cancel_typeahead_timer = |host: &mut dyn fret_ui::action::UiFocusActionHost| {
                if let Some(token) = typeahead_timer.get() {
                    host.push_effect(Effect::CancelTimer { token });
                    typeahead_timer.set(None);
                }
            };
            let clear_typeahead = |host: &mut dyn fret_ui::action::UiFocusActionHost| {
                typeahead.borrow_mut().clear();
                cancel_typeahead_timer(host);
            };

            match down.key {
                KeyCode::KeyA if primary && enable_row_selection && !single_row_selection => {
                    let leaf_keys = table_collect_leaf_keys(&meta);
                    if leaf_keys.is_empty() {
                        return false;
                    }

                    let all_selected = host
                        .models_mut()
                        .read(&state, |st| {
                            leaf_keys.iter().all(|k| st.row_selection.contains(k))
                        })
                        .ok()
                        .unwrap_or(false);

                    let _ = host.models_mut().update(&state, move |st| {
                        if down.modifiers.shift {
                            st.row_selection.clear();
                            return;
                        }

                        if all_selected {
                            for k in &leaf_keys {
                                st.row_selection.remove(k);
                            }
                        } else {
                            for k in &leaf_keys {
                                st.row_selection.insert(*k);
                            }
                        }
                    });
                    clear_typeahead(host);
                    host.request_redraw(action_cx.window);
                    true
                }
                KeyCode::ArrowDown
                | KeyCode::ArrowUp
                | KeyCode::Home
                | KeyCode::End
                | KeyCode::PageDown
                | KeyCode::PageUp => {
                    if primary {
                        return false;
                    }

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

                        if down.modifiers.shift {
                            if enable_row_selection
                                && let Some(m) = meta.get(next)
                                && m.kind == TableNavRowKind::Leaf
                            {
                                let anchor_key =
                                    anchor_index.get().unwrap_or(meta[current].row_key);
                                if anchor_index.get().is_none() {
                                    anchor_index.set(Some(anchor_key));
                                }
                                let anchor = meta
                                    .iter()
                                    .position(|m| m.row_key == anchor_key)
                                    .unwrap_or(current);
                                let (a, b) = if anchor <= next {
                                    (anchor, next)
                                } else {
                                    (next, anchor)
                                };

                                let leaf_range: Vec<RowKey> = if single_row_selection {
                                    vec![m.row_key]
                                } else {
                                    table_collect_leaf_keys_in_range(&meta, a, b)
                                };

                                if !leaf_range.is_empty() {
                                    let _ = host.models_mut().update(&state, move |st| {
                                        st.row_selection.clear();
                                        st.row_selection.extend(leaf_range.iter().copied());
                                    });
                                }
                            }
                        } else {
                            anchor_index.set(Some(meta[next].row_key));
                        }

                        clear_typeahead(host);
                        host.request_redraw(action_cx.window);
                    }
                    true
                }
                KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                    if primary {
                        return false;
                    }

                    let Some(meta) = meta.get(current).cloned() else {
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
                            clear_typeahead(host);
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
                            anchor_index.set(Some(row_key));
                            clear_typeahead(host);
                            host.request_redraw(action_cx.window);
                            true
                        }
                    }
                }
                _ => {
                    if primary {
                        return false;
                    }
                    if down.repeat {
                        return false;
                    }
                    let Some(input) = fret_core::keycode_to_ascii_lowercase(down.key) else {
                        return false;
                    };

                    typeahead.borrow_mut().push_char(input, 0);
                    let typeahead_buf = typeahead.borrow();
                    let Some(query) = typeahead_buf.query(0) else {
                        return false;
                    };

                    let labels: Vec<Arc<str>> = meta.iter().map(|m| m.label.clone()).collect();
                    let disabled: Vec<bool> =
                        meta.iter().map(|m| m.label.trim().is_empty()).collect();

                    let next = match_prefix_arc_str(&labels, &disabled, query, Some(current), true);
                    if let Some(next) = next
                        && next != current
                    {
                        active_index.set(Some(next));
                        anchor_index.set(Some(meta[next].row_key));
                        *active_command.borrow_mut() = None;
                        vertical_scroll.scroll_to_item(next, ScrollStrategy::Nearest);
                    }

                    cancel_typeahead_timer(host);
                    let token = host.next_timer_token();
                    typeahead_timer.set(Some(token));
                    host.push_effect(Effect::SetTimer {
                        window: Some(action_cx.window),
                        token,
                        after: TABLE_TYPEAHEAD_TIMEOUT,
                        repeat: None,
                    });

                    host.request_redraw(action_cx.window);
                    true
                }
            }
        })
    };

    let list = cx.semantics_with_id(
        SemanticsProps {
            role: SemanticsRole::List,
            focusable: true,
            ..Default::default()
        },
        |cx, list_id| {
            cx.key_on_key_down_for(list_id, key_handler.clone());
            if let Some(copy_text_at) = copy_text_at.clone() {
                let copy_text_for_command = copy_text_at.clone();
                let state_for_command = state.clone();
                let row_meta_for_command = row_meta.clone();
                cx.command_on_command_for(
                    list_id,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return false;
                        }
                        let meta = row_meta_for_command.borrow().clone();
                        let selected_keys = host
                            .models_mut()
                            .read(&state_for_command, |st| st.row_selection.clone())
                            .ok()
                            .unwrap_or_default();
                        let models = host.models_mut();
                        let mut lines = Vec::new();
                        if !selected_keys.is_empty() {
                            for m in meta.iter() {
                                if m.kind != TableNavRowKind::Leaf {
                                    continue;
                                }
                                if !selected_keys.contains(&m.row_key) {
                                    continue;
                                }
                                if let Some(data_index) = m.data_index
                                    && let Some(text) = (copy_text_for_command)(&*models, data_index)
                                {
                                    lines.push(text);
                                }
                            }
                        }

                        if lines.is_empty() {
                            return true;
                        }
                        let token = host.next_clipboard_token();
                        host.push_effect(Effect::ClipboardWriteText {
                            window: acx.window,
                            token,
                            text: lines.join("\n"),
                        });
                        true
                    }),
                );

                let state_for_availability = state.clone();
                let row_meta_for_availability = row_meta.clone();
                cx.command_on_command_availability_for(
                    list_id,
                    Arc::new(move |host, acx, command| {
                        if command.as_str() != "edit.copy" {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.focus_in_subtree {
                            return fret_ui::CommandAvailability::NotHandled;
                        }
                        if !acx.input_ctx.caps.clipboard.text.write {
                            return fret_ui::CommandAvailability::Blocked;
                        }

                        let meta = row_meta_for_availability.borrow().clone();
                        let selected_keys = host
                            .models_mut()
                            .read(&state_for_availability, |st| st.row_selection.clone())
                            .ok()
                            .unwrap_or_default();
                        if !selected_keys.is_empty() {
                            for m in meta.iter() {
                                if m.kind == TableNavRowKind::Leaf && selected_keys.contains(&m.row_key) {
                                    return fret_ui::CommandAvailability::Available;
                                }
                            }
                            return fret_ui::CommandAvailability::Blocked;
                        }

                        fret_ui::CommandAvailability::Blocked
                    }),
                );
            }
            {
                let typeahead = typeahead.clone();
                let typeahead_timer = typeahead_timer.clone();
                cx.timer_on_timer_for(
                    list_id,
                    Arc::new(move |host, action_cx, token| {
                        if typeahead_timer.get() != Some(token) {
                            return false;
                        }
                        typeahead_timer.set(None);
                        typeahead.borrow_mut().clear();
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
            }
            let focus_ring = RingStyle {
                placement: RingPlacement::Inset,
                width: Px(2.0),
                offset: Px(0.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(radius),
            };
            vec![cx.container(
                ContainerProps {
                    layout: table_clip_fill_layout(),
                    background: Some(table_bg),
                    border: if props.draw_frame {
                        Edges::all(Px(1.0))
                    } else {
                        Edges::all(Px(0.0))
                    },
                    border_color: if props.draw_frame { Some(border) } else { None },
                    focus_border_color: if props.draw_frame { Some(ring) } else { None },
                    focus_ring: Some(focus_ring),
                    focus_within: true,
                    corner_radii: if props.draw_frame {
                        Corners::all(radius)
                    } else {
                        Corners::all(Px(0.0))
                    },
                    ..Default::default()
                },
                |cx| {
                    vec![ui::v_flex(|cx| {
                                    let debug_header_row_test_id = debug_header_row_test_id.clone();
                                    let debug_header_cell_test_id_prefix =
                                        debug_header_cell_test_id_prefix.clone();
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
                                                    width: Length::Fill,
                                                    height: Length::Px(header_h),
                                                    min_height: Some(Length::Px(header_h)),
                                                    max_height: Some(Length::Px(header_h)),
                                                    ..Default::default()
                                                },
                                                flex: fret_ui::element::FlexItemStyle {
                                                    shrink: 0.0,
                                                    basis: Length::Px(header_h),
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
                                                    let column_width_by_id =
                                                        column_width_by_id.clone();
                                                    let debug_header_cell_test_id_prefix =
                                                        debug_header_cell_test_id_prefix.clone();
                                                    let row = ui::h_row(|cx| {
                                                            let out: Vec<AnyElement> = cols.iter()
                                                                .map(|col| {
                                                                    let sort_state = sort_for_column(
                                                                        &state_value.sorting,
                                                                        &col.id,
                                                                    );

                                                                    let col_w = column_width_by_id
                                                                        .get(&col.id)
                                                                        .copied()
                                                                        .unwrap_or(Px(col.size));

                                                                    let cell_props = ContainerProps {
                                                                        padding: Edges::all(Px(0.0)).into(),
                                                                        border: if props.optimize_grid_lines {
                                                                            Edges::default()
                                                                        } else {
                                                                            Edges {
                                                                                right: if props
                                                                                    .enable_column_resizing
                                                                                    && col.enable_resizing
                                                                                {
                                                                                    Px(0.0)
                                                                                } else {
                                                                                    Px(1.0)
                                                                                },
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
                                                                                min_width: Some(Length::Px(col_w)),
                                                                                max_width: Some(Length::Px(col_w)),
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

                                                                    let hoisted_test_id =
                                                                        Rc::new(RefCell::new(None));
                                                                    let hoisted_test_id_for_cell =
                                                                        hoisted_test_id.clone();
                                                                    let explicit_test_id =
                                                                        debug_header_cell_test_id_prefix
                                                                            .as_ref()
                                                                            .map(|prefix| {
                                                                                Arc::<str>::from(format!(
                                                                                    "{prefix}{id}",
                                                                                    id = col.id.as_ref()
                                                                                ))
                                                                            });
                                                                    let header_cell =
                                                                        cx.container(cell_props, |cx| {
                                                                        let mut out = Vec::new();

                                                                        out.push(ui::h_row(|cx| {
                                                                                let mut pieces = Vec::new();

                                                                                let enabled = props.enable_sorting
                                                                                    && col.enable_sorting
                                                                                    && (col.sort_cmp.is_some()
                                                                                        || col.sort_value.is_some());
                                                                                let col_id = col.id.clone();
                                                                                let state_model =
                                                                                    state.clone();
                                                                                let sort_options =
                                                                                    TableOptions {
                                                                                        enable_sorting: props
                                                                                            .enable_sorting,
                                                                                        ..TableOptions::default()
                                                                                    };
                                                                                let sort_toggle_column =
                                                                                    SortToggleColumn {
                                                                                        id: col_id.clone(),
                                                                                        enable_sorting: col
                                                                                            .enable_sorting,
                                                                                        enable_multi_sort: col
                                                                                            .enable_multi_sort,
                                                                                        sort_desc_first: col
                                                                                            .sort_desc_first,
                                                                                        has_sort_value_source: col
                                                                                            .sort_cmp
                                                                                            .is_some()
                                                                                            || col.sort_value.is_some(),
                                                                                    };
                                                                                let auto_sort_dir_desc = col
                                                                                    .sort_value
                                                                                    .as_ref()
                                                                                    .and_then(|f| {
                                                                                        data.first()
                                                                                            .map(|r| f(r))
                                                                                    })
                                                                                    .map(|v| {
                                                                                        !matches!(
                                                                                            v,
                                                                                            TanStackValue::String(_)
                                                                                        )
                                                                                    })
                                                                                    .unwrap_or(false);

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
                                                                                            let state_model_for_pointer =
                                                                                                state_model.clone();
                                                                                            let sort_toggle_column_for_pointer =
                                                                                                sort_toggle_column.clone();
                                                                                            let sort_options_for_pointer =
                                                                                                sort_options;
                                                                                            cx.pressable_on_pointer_up(Arc::new(
                                                                                                move |host, _acx, up| {
                                                                                                    if !up.is_click
                                                                                                        || up.button
                                                                                                            != fret_core::MouseButton::Left
                                                                                                    {
                                                                                                        return PressablePointerUpResult::Continue;
                                                                                                    }

                                                                                                    let multi =
                                                                                                        up.modifiers.shift;
                                                                                                    let _ = host
                                                                                                        .update_model(
                                                                                                            &state_model_for_pointer,
                                                                                                            |st| {
                                                                                                                toggle_sorting_state_handler_tanstack(
                                                                                                                    &mut st.sorting,
                                                                                                                    &sort_toggle_column_for_pointer,
                                                                                                                    sort_options_for_pointer,
                                                                                                                    multi,
                                                                                                                    auto_sort_dir_desc,
                                                                                                                );
                                                                                                                st.pagination.page_index = 0;
                                                                                                            },
                                                                                                        );
                                                                                                    host.notify(_acx);
                                                                                                    PressablePointerUpResult::SkipActivate
                                                                                                },
                                                                                            ));

                                                                                            cx.pressable_update_model(
                                                                                                &state_model,
                                                                                                move |st| {
                                                                                                    toggle_sorting_state_handler_tanstack(
                                                                                                        &mut st.sorting,
                                                                                                        &sort_toggle_column,
                                                                                                        sort_options,
                                                                                                        false,
                                                                                                        auto_sort_dir_desc,
                                                                                                    );
                                                                                                    st.pagination.page_index = 0;
                                                                                                },
                                                                                            );
                                                                                        }

                                                                                        vec![cx.container(
                                                                                            ContainerProps {
                                                                                                padding: Edges::symmetric(
                                                                                                    cell_px, cell_py,
                                                                                                )
                                                                                                .into(),
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
                                                                                            |cx| {
                                                                                                let items =
                                                                                                    render_header_cell(
                                                                                                        cx,
                                                                                                        col,
                                                                                                        sort_state,
                                                                                                    );
                                                                                                let mut children =
                                                                                                    collect_children(
                                                                                                        cx, items,
                                                                                                    );
                                                                                                *hoisted_test_id_for_cell
                                                                                                    .borrow_mut() =
                                                                                                    table_wrapper_test_id(
                                                                                                        &mut children,
                                                                                                        explicit_test_id.clone(),
                                                                                                    );
                                                                                                children
                                                                                            },
                                                                                        )]
                                                                                    },
                                                                                ));

                                                                                if props.enable_column_resizing
                                                                                    && col.enable_resizing
                                                                                {
                                                                                    let col_id = col.id.clone();
                                                                                    let state_model = state.clone();
                                                                                    let default_w = Px(col.size);
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
                                                                                                        top: Some(Px(0.0)).into(),
                                                                                                        right: Some(Px(-delta - 1.0)).into(),
                                                                                                        bottom: Some(Px(0.0)).into(),
                                                                                                        left: None.into(),
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
                                                                                                    top: Some(Px(0.0)).into(),
                                                                                                    right: Some(Px(0.0)).into(),
                                                                                                    bottom: Some(Px(0.0)).into(),
                                                                                                    left: None.into(),
                                                                                                },
                                                                                                ..Default::default()
                                                                                            },
                                                                                            enabled: true,
                                                                                            capture_phase_pointer_moves: false,
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
                                                                                                            start,
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
                                                                                                        if st
                                                                                                            .column_sizing_info
                                                                                                            .is_resizing_column
                                                                                                            .as_deref()
                                                                                                            != Some(col_id_up.as_ref())
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
                                                                                            vec![ui::h_row(|cx| {
                                                                                                [cx.container(
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
                                                                                            })
                                                                                            .gap(Space::N0)
                                                                                            .justify_end()
                                                                                            .items_stretch()
                                                                                            .into_element(cx)]
                                                                                        },
                                                                                    ));
                                                                                }

                                                                                pieces
                                                                            })
                                                                            .layout(
                                                                                LayoutRefinement::default()
                                                                                    .size_full()
                                                                                    .relative(),
                                                                            )
                                                                            .gap(Space::N0)
                                                                            .justify_start()
                                                                            .items_center()
                                                                            .into_element(cx));

                                                                                out
                                                                    });

                                                                    if let Some(test_id) =
                                                                        hoisted_test_id.borrow_mut().take()
                                                                    {
                                                                        header_cell.test_id(test_id)
                                                                    } else {
                                                                        header_cell
                                                                    }
                                                                })
                                                                .collect();

                                                            out
                                                        })
                                                        .gap(Space::N0)
                                                        .justify_start()
                                                        .items_center()
                                                        .into_element(cx);

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

                                            vec![ui::h_row(|cx| {
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
                                                    [left, center, right]
                                            })
                                            .gap(Space::N0)
                                            .justify_start()
                                            .items_stretch()
                                            .into_element(cx)]
                                        },
                                    );
                                    let header = if let Some(test_id) = debug_header_row_test_id {
                                        header.test_id(test_id)
                                    } else {
                                        header
                                    };

                                    let debug_row_test_id_prefix = debug_row_test_id_prefix.clone();
                                    let body = cx.virtual_list_keyed_with_layout(
                                        {
                                            let mut layout = LayoutStyle::default();
                                            layout.size.width = Length::Fill;
                                            layout.flex.grow = 1.0;
                                            layout.flex.shrink = 1.0;
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
                                                        let anchor_index = anchor_index.clone();
                                                        let active_element = active_element.clone();
                                                        let active_command = active_command.clone();
                                                        let typeahead = typeahead.clone();
                                                        let typeahead_timer = typeahead_timer.clone();
                                                        let focus_target = list_id;
                                                        let row_test_id = debug_row_test_id_prefix
                                                            .as_ref()
                                                            .map(|prefix| {
                                                                Arc::<str>::from(format!(
                                                                    "{prefix}{id}",
                                                                    id = row_key.0
                                                                ))
                                                            });

                                                        return cx.pressable_with_id(
                                                            PressableProps {
                                                                enabled,
                                                                focusable: false,
                                                                a11y: PressableA11y {
                                                                    role: Some(
                                                                        SemanticsRole::ListItem,
                                                                    ),
                                                                    expanded: Some(expanded),
                                                                    test_id: row_test_id,
                                                                    ..Default::default()
                                                                }
                                                                .with_collection_position(i, set_size),
                                                                ..Default::default()
                                                            },
															|cx, st, id| {
																let active_index_for_pointer =
																	active_index.clone();
																let anchor_index_for_pointer =
																	anchor_index.clone();
																let active_command_for_pointer =
																	active_command.clone();
																let typeahead_for_pointer =
																	typeahead.clone();
																let typeahead_timer_for_pointer =
																	typeahead_timer.clone();
                                                            cx.pressable_on_pointer_down(Arc::new(
                                                                move |host, action_cx, down| {
                                                                    active_index_for_pointer.set(Some(i));
                                                                    let next_anchor = if down.modifiers.shift {
                                                                        anchor_index_for_pointer
                                                                            .get()
                                                                            .or(Some(row_key))
                                                                    } else {
                                                                        Some(row_key)
                                                                    };
                                                                    anchor_index_for_pointer
                                                                        .set(next_anchor);
                                                                    typeahead_for_pointer
                                                                        .borrow_mut()
                                                                        .clear();
                                                                    if let Some(token) =
                                                                        typeahead_timer_for_pointer.get()
                                                                    {
                                                                        host.push_effect(Effect::CancelTimer { token });
                                                                        typeahead_timer_for_pointer.set(None);
                                                                    }
                                                                    *active_command_for_pointer.borrow_mut() = None;
                                                                    host.request_redraw(action_cx.window);
                                                                    PressablePointerDownResult::Continue
                                                                },
                                                            ));
																cx.pressable_add_on_pointer_up(Arc::new(
																	move |host, action_cx, up| {
																		if up.button
																			!= fret_core::MouseButton::Left
																			|| !up.is_click
																		{
																			return PressablePointerUpResult::Continue;
																		}
																		host.request_focus(focus_target);
																		host.request_redraw(action_cx.window);
																		PressablePointerUpResult::Continue
																	},
																));

																if active_index.get() == Some(i) {
																	active_element.set(Some(id));
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
                                                                    Some(row_active_bg)
                                                                } else if st.hovered {
                                                                    Some(row_hover_bg)
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
                                                                            size: fret_ui::element::SizeStyle {
                                                                                width: Length::Fill,
                                                                                height: body_row_height,
                                                                                ..Default::default()
                                                                            },
                                                                            position:
                                                                                fret_ui::element::PositionStyle::Relative,
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    |cx| {
                                                                        let mut out = Vec::new();
                                                                        if is_active {
                                                                            out.push(cx.container(
                                                                                ContainerProps {
                                                                                    background: Some(ring),
                                                                                    layout: LayoutStyle {
                                                                                        size: fret_ui::element::SizeStyle {
                                                                                            width: Length::Px(Px(2.0)),
                                                                                            height: Length::Fill,
                                                                                            ..Default::default()
                                                                                        },
                                                                                        position:
                                                                                            fret_ui::element::PositionStyle::Absolute,
                                                                                        inset: fret_ui::element::InsetStyle {
                                                                                            top: Some(Px(0.0)).into(),
                                                                                            right: None.into(),
                                                                                            bottom: Some(Px(0.0)).into(),
                                                                                            left: Some(Px(0.0)).into(),
                                                                                        },
                                                                                        ..Default::default()
                                                                                    },
                                                                                    ..Default::default()
                                                                                },
                                                                                |_| Vec::new(),
                                                                            ));
                                                                        }
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
                                                                                let column_width_by_id =
                                                                                    column_width_by_id.clone();
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
                                                                                                    column_width_by_id
                                                                                                        .get(&col.id)
                                                                                                        .copied()
                                                                                                        .unwrap_or(Px(col.size))
                                                                                                })
                                                                                                .collect();
                                                                                            let background_row = ui::h_row(|cx| {
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
                                                                                                                        min_width: Some(Length::Px(col_w)),
                                                                                                                        max_width: Some(Length::Px(col_w)),
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
                                                                                                    .collect::<Vec<_>>()
                                                                                            })
                                                                                            .gap(Space::N0)
                                                                                            .justify_start()
                                                                                            .items_stretch()
                                                                                            .into_element(cx);

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
                                                                                                            top: Some(Px(0.0)).into(),
                                                                                                            right: Some(Px(0.0)).into(),
                                                                                                            bottom: Some(Px(0.0)).into(),
                                                                                                            left: Some(Px(0.0)).into(),
                                                                                                        },
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    ..Default::default()
                                                                                                },
                                                                                                |cx| {
                                                                                                    vec![ui::h_row(|cx| {
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
                                                                                                                            padding: padding.into(),
                                                                                                                           layout: LayoutStyle {
                                                                                                                                size: fret_ui::element::SizeStyle {
                                                                                                                                    width: Length::Px(col_w),
                                                                                                                                    min_width: Some(Length::Px(col_w)),
                                                                                                                                    max_width: Some(Length::Px(col_w)),
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
                                                                                                            .collect::<Vec<_>>()
                                                                                                    })
                                                                                                    .gap(Space::N0)
                                                                                                    .justify_start()
                                                                                                    .items_center()
                                                                                                    .into_element(cx)]
                                                                                                },
                                                                                            );

                                                                                            vec![background_row, content_overlay]
                                                                                        },
                                                                                    )
                                                                                } else {
                                                                                    ui::h_row(|cx| {
                                                                                            cols.iter()
                                                                                                .map(|col| {
                                                                                                    let col_w = column_width_by_id
                                                                                                        .get(&col.id)
                                                                                                        .copied()
                                                                                                        .unwrap_or(Px(col.size));

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
                                                                                                            padding: padding.into(),
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
                                                                                                            min_width: Some(Length::Px(col_w)),
                                                                                                            max_width: Some(Length::Px(col_w)),
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
                                                                                                .collect::<Vec<_>>()
                                                                                    })
                                                                                    .gap(Space::N0)
                                                                                    .justify_start()
                                                                                    .items_center()
                                                                                    .into_element(cx)
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

                                                                        out.push(ui::h_row(|cx| {
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
                                                                        })
                                                                        .gap(Space::N0)
                                                                        .justify_start()
                                                                        .items_stretch()
                                                                        .into_element(cx));
                                                                        out
                                                                    },
                                                                )]
                                                            },
                                                        );
                                                    }
                                                };

                                            let data_row = Row {
                                                id: RowId::new(row_key.0.to_string()),
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
                                            let row_test_id = debug_row_test_id_prefix
                                                .as_ref()
                                                .map(|prefix| {
                                                    Arc::<str>::from(format!(
                                                        "{prefix}{id}",
                                                        id = data_row.key.0
                                                    ))
                                                });

                                            let active_index = active_index.clone();
                                            let anchor_index = anchor_index.clone();
                                            let active_element = active_element.clone();
                                            let active_command = active_command.clone();
                                            let typeahead = typeahead.clone();
                                            let typeahead_timer = typeahead_timer.clone();
                                            let focus_target = list_id;

                                            cx.pressable_with_id(
                                                PressableProps {
                                                    enabled,
                                                    focusable: false,
                                                    a11y: PressableA11y {
                                                        role: Some(SemanticsRole::ListItem),
                                                        selected: is_selected,
                                                        test_id: row_test_id,
                                                        ..Default::default()
                                                    }
                                                    .with_collection_position(i, set_size),
                                                    ..Default::default()
                                                },
                                                |cx, st, id| {
                                                    let active_index_for_pointer =
                                                        active_index.clone();
                                                    let anchor_index_for_pointer_down =
                                                        anchor_index.clone();
                                                    let anchor_index_for_pointer_up =
                                                        anchor_index.clone();
                                                    let row_meta_for_pointer = row_meta.clone();
                                                    let typeahead_for_pointer = typeahead.clone();
                                                    let typeahead_timer_for_pointer =
                                                        typeahead_timer.clone();
                                                    let state_model_for_pointer = state.clone();
                                                    let row_key_for_pointer = data_row.key;

                                                    cx.pressable_on_pointer_down(Arc::new(
                                                        move |host, action_cx, down| {
                                                            active_index_for_pointer.set(Some(i));
                                                            let next_anchor = if down.modifiers.shift {
                                                                anchor_index_for_pointer_down
                                                                    .get()
                                                                    .or(Some(row_key_for_pointer))
                                                            } else {
                                                                Some(row_key_for_pointer)
                                                            };
                                                            anchor_index_for_pointer_down
                                                                .set(next_anchor);
                                                            typeahead_for_pointer
                                                                .borrow_mut()
                                                                .clear();
                                                            if let Some(token) =
                                                                typeahead_timer_for_pointer.get()
                                                            {
                                                                host.push_effect(
                                                                    Effect::CancelTimer { token },
                                                                );
                                                                typeahead_timer_for_pointer
                                                                    .set(None);
                                                            }
                                                            host.request_redraw(action_cx.window);
                                                            PressablePointerDownResult::Continue
                                                        },
                                                    ));

                                                    cx.pressable_add_on_pointer_up(Arc::new(
                                                        move |host, action_cx, up| {
															if up.button
																!= fret_core::MouseButton::Left
																|| !up.is_click
															{
																return PressablePointerUpResult::Continue;
															}
															host.request_focus(focus_target);
                                                            let pointer_row_selection_enabled = props.enable_row_selection
                                                                && props.pointer_row_selection;
															if pointer_row_selection_enabled {
																let policy =
																	props.pointer_row_selection_policy;
																let modifiers = up.modifiers;
																let row_key = row_key_for_pointer;
																let single = props.single_row_selection;
															let meta = row_meta_for_pointer
																	.borrow()
																	.clone();
																let range_keys = if policy
																	== PointerRowSelectionPolicy::ListLike
																	&& !single
																	&& modifiers.shift
																{
                                                                    let anchor_key = anchor_index_for_pointer_up
                                                                        .get()
                                                                        .unwrap_or(row_key);
                                                                    let anchor = meta
                                                                        .iter()
                                                                        .position(|m| m.row_key == anchor_key)
                                                                        .unwrap_or(i);
																	let (a, b) = if anchor <= i {
																		(anchor, i)
																	} else {
																		(i, anchor)
																	};
																	let keys = if single {
																		vec![row_key]
																	} else {
																		table_collect_leaf_keys_in_range(
																			&meta, a, b,
																		)
																	};
																	(!keys.is_empty()).then_some(keys)
																} else {
																	None
																};

																let _ = host.models_mut().update(
																	&state_model_for_pointer,
																	move |st| match policy {
																		PointerRowSelectionPolicy::Toggle => {
																			let selected =
																				st.row_selection
																					.contains(&row_key);
																			if single {
																				st.row_selection.clear();
																			}
																			if selected {
																				st.row_selection
																					.remove(&row_key);
																			} else {
																				st.row_selection
																					.insert(row_key);
																			}
																		}
																		PointerRowSelectionPolicy::ListLike => {
																			if let Some(range_keys) =
																				range_keys.as_ref()
																			{
																				if modifiers.ctrl
																					|| modifiers.meta
																				{
																					st.row_selection.extend(
																						range_keys
																							.iter()
																							.copied(),
																					);
																				} else {
																					st.row_selection.clear();
																					st.row_selection.extend(
																						range_keys
																							.iter()
																							.copied(),
																					);
																				}
																			} else if !single
																				&& (modifiers.ctrl
																					|| modifiers.meta)
																			{
																				if st.row_selection
																					.contains(&row_key)
																				{
																					st.row_selection
																						.remove(&row_key);
																				} else {
																					st.row_selection
																						.insert(row_key);
																				}
																			} else {
																				st.row_selection.clear();
																				st.row_selection
																					.insert(row_key);
																			}
																		}
																	},
																);

																	let next_anchor = if policy
																		== PointerRowSelectionPolicy::ListLike
																		&& modifiers.shift
																{
																	anchor_index_for_pointer_up
																		.get()
																		.or(Some(row_key))
																} else {
																	Some(row_key)
																};
																anchor_index_for_pointer_up
																	.set(next_anchor);
															}
															host.request_redraw(action_cx.window);
                                                            // When pointer-driven row selection is enabled, a click should not
                                                            // also activate the row command (avoid "selection + activate" conflicts).
															PressablePointerUpResult::SkipActivate
	                                                        },
	                                                    ));

														if active_index.get() == Some(i) {
															active_element.set(Some(id));
															*active_command.borrow_mut() = cmd.clone();
														}
													cx.pressable_dispatch_command_if_enabled_opt(cmd.clone());

													let is_active = active_index.get() == Some(i);
													let bg = if is_selected || (enabled && st.pressed) {
														Some(row_active_bg)
                                                    } else if enabled && st.hovered {
                                                        Some(row_hover_bg)
                                                    } else {
                                                        None
                                                    };

                                                        vec![cx.container(
                                                            ContainerProps {
                                                                background: bg,
                                                                layout: LayoutStyle {
                                                                    size: fret_ui::element::SizeStyle {
                                                                        width: Length::Fill,
                                                                        height: body_row_height,
                                                                        ..Default::default()
                                                                    },
                                                                    position:
                                                                        fret_ui::element::PositionStyle::Relative,
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            },
                                                            |cx| {
                                                            let mut out = Vec::new();
                                                            if is_active {
                                                                out.push(cx.container(
                                                                    ContainerProps {
                                                                        background: Some(ring),
                                                                        layout: LayoutStyle {
                                                                            size:
                                                                                fret_ui::element::SizeStyle {
                                                                                    width: Length::Px(Px(2.0)),
                                                                                    height: Length::Fill,
                                                                                    ..Default::default()
                                                                                },
                                                                            position:
                                                                                fret_ui::element::PositionStyle::Absolute,
                                                                            inset: fret_ui::element::InsetStyle {
                                                                                top: Some(Px(0.0)).into(),
                                                                                right: None.into(),
                                                                                bottom: Some(Px(0.0)).into(),
                                                                                left: Some(Px(0.0)).into(),
                                                                            },
                                                                            ..Default::default()
                                                                        },
                                                                        ..Default::default()
                                                                    },
                                                                    |_| Vec::new(),
                                                                ));
                                                            }
                                                            let mut render_row_group =
                                                                |cx: &mut ElementContext<'_, H>,
                                                                 cols: &[&ColumnDef<TData>],
                                                                 scroll_x: Option<ScrollHandle>| {
                                                                    let column_width_by_id =
                                                                        column_width_by_id.clone();
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
                                                                                        column_width_by_id
                                                                                            .get(&col.id)
                                                                                            .copied()
                                                                                            .unwrap_or(Px(col.size))
                                                                                    })
                                                                                    .collect();
                                                                                let background_row = ui::h_row(|cx| {
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
                                                                                                        layout: table_fixed_column_fill_layout(col_w),
                                                                                                        ..Default::default()
                                                                                                    },
                                                                                                    |_| Vec::new(),
                                                                                                )
                                                                                            })
                                                                                            .collect::<Vec<_>>()
                                                                                })
                                                                                .gap(Space::N0)
                                                                                .justify_start()
                                                                                .items_stretch()
                                                                                .into_element(cx);

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
                                                                                                top: Some(Px(0.0)).into(),
                                                                                                right: Some(Px(0.0)).into(),
                                                                                                bottom: Some(Px(0.0)).into(),
                                                                                                left: Some(Px(0.0)).into(),
                                                                                            },
                                                                                            ..Default::default()
                                                                                        },
                                                                                        ..Default::default()
                                                                                    },
                                                                                    |cx| {
                                                                                        vec![ui::h_row(|cx| {
                                                                                                cols.iter()
                                                                                                    .zip(col_widths.iter().copied())
                                                                                                    .map(|(col, col_w)| {
                                                                                                        let hoisted_test_id =
                                                                                                            Rc::new(RefCell::new(None));
                                                                                                        let hoisted_test_id_for_cell =
                                                                                                            hoisted_test_id.clone();
                                                                                                        let explicit_test_id =
                                                                                                            debug_row_test_id_prefix
                                                                                                                .as_ref()
                                                                                                                .map(|prefix| {
                                                                                                                    Arc::<str>::from(format!(
                                                                                                                        "{prefix}{row}-cell-{col}",
                                                                                                                        row = data_row.key.0,
                                                                                                                        col = col.id.as_ref()
                                                                                                                    ))
                                                                                                                });
                                                                                                        let cell =
                                                                                                            cx.container(
                                                                                                            ContainerProps {
                                                                                                                padding: Edges::symmetric(
                                                                                                                    cell_px,
                                                                                                                    cell_py,
                                                                                                                )
                                                                                                                .into(),
                                                                                                                layout:
                                                                                                                    table_fixed_column_clip_fill_layout(
                                                                                                                        col_w,
                                                                                                                    ),
                                                                                                                ..Default::default()
                                                                                                            },
                                                                                                            |cx| {
                                                                                                                let items =
                                                                                                                    render_cell(
                                                                                                                        cx,
                                                                                                                        &data_row,
                                                                                                                        col,
                                                                                                                    );
                                                                                                                let mut children =
                                                                                                                    collect_children(
                                                                                                                        cx, items,
                                                                                                                    );
                                                                                                                *hoisted_test_id_for_cell
                                                                                                                    .borrow_mut() =
                                                                                                                    table_wrapper_test_id(
                                                                                                                        &mut children,
                                                                                                                        explicit_test_id
                                                                                                                            .clone(),
                                                                                                                    );
                                                                                                                let content = ui::h_row(
                                                                                                                    move |_cx| {
                                                                                                                        children
                                                                                                                    },
                                                                                                                )
                                                                                                                .layout(
                                                                                                                    LayoutRefinement::default()
                                                                                                                        .w_full()
                                                                                                                        .h_full(),
                                                                                                                )
                                                                                                                .gap(Space::N0)
                                                                                                                .justify_start()
                                                                                                                .items_center()
                                                                                                                .into_element(cx);
                                                                                                                vec![content]
                                                                                                            },
                                                                                                        );
                                                                                                        if let Some(test_id) =
                                                                                                            hoisted_test_id.borrow_mut().take()
                                                                                                        {
                                                                                                            cell.test_id(test_id)
                                                                                                        } else {
                                                                                                            cell
                                                                                                        }
                                                                                                    })
                                                                                                    .collect::<Vec<_>>()
                                                                                        })
                                                                                        .gap(Space::N0)
                                                                                        .justify_start()
                                                                                        .items_center()
                                                                                        .into_element(cx)]
                                                                                    },
                                                                                );

                                                                                vec![background_row, content_overlay]
                                                                            },
                                                                        )
                                                                    } else {
                                                                        ui::h_row(|cx| {
                                                                                    cols.iter()
                                                                                        .map(|col| {
                                                                                            let col_w = column_width_by_id
                                                                                                .get(&col.id)
                                                                                                .copied()
                                                                                                .unwrap_or(Px(col.size));
                                                                                            let hoisted_test_id =
                                                                                                Rc::new(RefCell::new(None));
                                                                                            let hoisted_test_id_for_cell =
                                                                                                hoisted_test_id.clone();
                                                                                            let explicit_test_id =
                                                                                                debug_row_test_id_prefix
                                                                                                    .as_ref()
                                                                                                    .map(|prefix| {
                                                                                                        Arc::<str>::from(format!(
                                                                                                            "{prefix}{row}-cell-{col}",
                                                                                                            row = data_row.key.0,
                                                                                                            col = col.id.as_ref()
                                                                                                        ))
                                                                                                    });
                                                                                            let cell = cx.container(
                                                                                                ContainerProps {
                                                                                                    padding: Edges::symmetric(
                                                                                                        cell_px, cell_py,
                                                                                                    )
                                                                                                    .into(),
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
                                                                                                    layout:
                                                                                                        table_fixed_column_clip_fill_layout(
                                                                                                            col_w,
                                                                                                        ),
                                                                                                ..Default::default()
                                                                                            },
                                                                                            |cx| {
                                                                                                let items =
                                                                                                    render_cell(
                                                                                                        cx,
                                                                                                        &data_row,
                                                                                                        col,
                                                                                                    );
                                                                                                let mut children =
                                                                                                    collect_children(
                                                                                                        cx, items,
                                                                                                    );
                                                                                                *hoisted_test_id_for_cell
                                                                                                    .borrow_mut() =
                                                                                                    table_wrapper_test_id(
                                                                                                        &mut children,
                                                                                                        explicit_test_id
                                                                                                            .clone(),
                                                                                                    );
                                                                                                let content = ui::h_row(
                                                                                                    move |_cx| {
                                                                                                        children
                                                                                                    },
                                                                                                )
                                                                                                .layout(
                                                                                                    LayoutRefinement::default()
                                                                                                        .w_full()
                                                                                                        .h_full(),
                                                                                                )
                                                                                                .gap(Space::N0)
                                                                                                .justify_start()
                                                                                                .items_center()
                                                                                                .into_element(cx);
                                                                                                vec![content]
                                                                                            },
                                                                                        );
                                                                                            if let Some(test_id) =
                                                                                                hoisted_test_id.borrow_mut().take()
                                                                                            {
                                                                                                cell.test_id(test_id)
                                                                                            } else {
                                                                                                cell
                                                                                            }
                                                                                    })
                                                                                    .collect::<Vec<_>>()
                                                                        })
                                                                        .gap(Space::N0)
                                                                        .justify_start()
                                                                        .items_center()
                                                                        .into_element(cx)
                                                                    };

                                                                    table_wrap_horizontal_scroll(
                                                                        cx, scroll_x, row,
                                                                    )
                                                                };

                                                            out.push(ui::h_row(|cx| {
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
                                                            })
                                                            .gap(Space::N0)
                                                            .justify_start()
                                                            .items_stretch()
                                                            .into_element(cx));
                                                            out
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
                        })
                    .h_full()
                    .into_element(cx)]
                },
            )]
        },
    );

    if let Some(active_element) = active_element.get() {
        // The active row element is discovered while the table body mounts. Keep the relationship
        // declarative here and let the semantics pass resolve it against the final mounted node
        // once the current frame commits.
        list.attach_semantics(
            SemanticsDecoration::default().active_descendant_element(active_element.0),
        )
    } else {
        list
    }
}
