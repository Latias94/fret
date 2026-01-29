use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, Color, FontId, FontWeight, Px, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, Overflow};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::table::{
    TableRowMeasureMode, TableViewOutput, TableViewProps, table_virtualized,
};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};

use fret_ui_headless::table::{ColumnDef, RowKey, TableState};

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn table_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.table.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.table.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn mixed_revision(a: u64, b: u64) -> u64 {
    // Cheap, deterministic mixing to avoid obvious collisions.
    a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

/// shadcn/ui `DataTable` backed by the TanStack-aligned headless engine (ADR 0101).
///
/// This is an integration surface:
/// - headless row model: filtering/sorting/pagination/visibility (future: sizing/pinning)
/// - UI: `Table` primitives + fixed header + virtualized body
///
/// Prefer `DataTable` for "business table" use-cases (toolbars, filters, pagination, column
/// visibility). For spreadsheet-scale density, prefer the canvas-backed [`crate::DataGrid`].
///
/// Notes (v0):
/// - row activation toggles selection by `RowKey` (flat tables; sub-row selection is deferred)
/// - header activation toggles single-column sorting (multi-sort key modifiers are deferred)
#[derive(Debug, Clone)]
pub struct DataTable {
    overscan: usize,
    row_height: Option<Px>,
    measure_rows: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    output: Option<Model<TableViewOutput>>,
}

impl Default for DataTable {
    fn default() -> Self {
        Self {
            overscan: 4,
            row_height: None,
            measure_rows: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            output: None,
        }
    }
}

impl DataTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }

    pub fn row_height(mut self, row_height: Px) -> Self {
        self.row_height = Some(row_height);
        self
    }

    /// Enables measured (variable-height) body rows.
    ///
    /// This is intended for content-driven row heights (e.g. wrapping Markdown).
    /// The default remains fixed-height rows for performance.
    pub fn measure_rows(mut self, enabled: bool) -> Self {
        self.measure_rows = enabled;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn output_model(mut self, output: Model<TableViewOutput>) -> Self {
        self.output = Some(output);
        self
    }

    /// A retained-host variant of [`Self::into_element`] that enables composable body rows under
    /// cache-root reuse (virt-003 / ADR 0192).
    ///
    /// Notes (v0):
    /// - supports fixed-height and measured (variable-height) rows
    /// - intended for perf/correctness harnesses; API stability is not guaranteed
    pub fn into_element_retained<H: UiHost + 'static, TData>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> AnyElement + 'static,
        debug_header_cell_test_id_prefix: Option<Arc<str>>,
        debug_row_test_id_prefix: Option<Arc<str>>,
    ) -> AnyElement
    where
        TData: 'static,
    {
        let DataTable {
            overscan,
            row_height,
            measure_rows,
            chrome,
            layout,
            output: _output,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let border = border_color(&theme);

        let state_revision = state.revision(&*cx.app).unwrap_or(0);
        let items_revision = mixed_revision(data_revision, state_revision);

        let columns: Arc<[ColumnDef<TData>]> = columns.into();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        cx.container(root_props, move |cx| {
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let get_row_key = Arc::new(get_row_key);
            let header_label = Arc::new(header_label);
            let cell_at = Arc::new(cell_at);

            let mut view_props = TableViewProps::default();
            view_props.overscan = overscan;
            view_props.row_height = row_height;
            view_props.row_measure_mode = if measure_rows {
                TableRowMeasureMode::Measured
            } else {
                TableRowMeasureMode::Fixed
            };
            view_props.enable_column_grouping = false;
            view_props.enable_column_resizing = false;
            view_props.draw_frame = false;

            let row_key_at = Arc::new(move |d: &TData, index: usize| (get_row_key)(d, index, None));

            vec![
                fret_ui_kit::declarative::table::table_virtualized_retained_v0(
                    cx,
                    data.clone(),
                    columns.clone(),
                    state.clone(),
                    &scroll_handle,
                    items_revision,
                    row_key_at,
                    None,
                    view_props,
                    header_label,
                    cell_at,
                    debug_header_cell_test_id_prefix,
                    debug_row_test_id_prefix,
                ),
            ]
        })
    }

    pub fn into_element<H: UiHost, TData>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: impl Into<Arc<[ColumnDef<TData>]>>,
        get_row_key: impl Fn(&TData, usize, Option<&RowKey>) -> RowKey + 'static,
        header_label: impl Fn(&ColumnDef<TData>) -> Arc<str> + 'static,
        cell_at: impl Fn(&mut ElementContext<'_, H>, &ColumnDef<TData>, &TData) -> AnyElement + 'static,
    ) -> AnyElement
    where
        TData: 'static,
    {
        let DataTable {
            overscan,
            row_height,
            measure_rows,
            chrome,
            layout,
            output,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let border = border_color(&theme);

        let state_revision = state.revision(&*cx.app).unwrap_or(0);
        let items_revision = mixed_revision(data_revision, state_revision);

        let columns: Arc<[ColumnDef<TData>]> = columns.into();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

            let header_style = TextStyle {
                weight: FontWeight::MEDIUM,
                ..table_text_style(&theme)
            };
            let header_fg = theme.color_required("foreground");
            let sort_fg = theme.color_required("muted-foreground");

            let get_row_key = Arc::new(get_row_key);
            let header_label = Arc::new(header_label);
            let cell_at = Arc::new(cell_at);

            let mut view_props = TableViewProps::default();
            view_props.overscan = overscan;
            view_props.row_height = row_height;
            view_props.row_measure_mode = if measure_rows {
                TableRowMeasureMode::Measured
            } else {
                TableRowMeasureMode::Fixed
            };
            view_props.enable_column_grouping = false;
            view_props.enable_column_resizing = false;
            view_props.draw_frame = false;

            let row_key_at = move |d: &TData, index: usize| (get_row_key)(d, index, None);

            let columns = columns.clone();
            let state = state.clone();
            let data = data.clone();
            vec![table_virtualized(
                cx,
                data.as_ref(),
                columns.as_ref(),
                state,
                &scroll_handle,
                items_revision,
                &row_key_at,
                None,
                view_props,
                |_row| None,
                move |cx, col, sort_state| {
                    let theme = Theme::global(&*cx.app).clone();
                    let label = (header_label)(col);
                    let style = header_style.clone();
                    let header_fg = header_fg;
                    let sort_fg = sort_fg;
                    vec![cx.flex(
                        FlexProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().w_full().h_full(),
                            ),
                            direction: Axis::Horizontal,
                            gap: Px(0.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            let mut pieces: Vec<AnyElement> = Vec::new();
                            {
                                let mut text = ui::label(cx, label.clone())
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .text_color(ColorRef::Color(header_fg))
                                    .nowrap();
                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }
                                pieces.push(text.into_element(cx));
                            }

                            if let Some(desc) = sort_state {
                                let indicator: Arc<str> =
                                    Arc::from(if desc { " ▼" } else { " ▲" });
                                let mut text = ui::label(cx, indicator)
                                    .text_size_px(style.size)
                                    .font_weight(style.weight)
                                    .text_color(ColorRef::Color(sort_fg))
                                    .nowrap();
                                if let Some(line_height) = style.line_height {
                                    text = text.line_height_px(line_height);
                                }
                                if let Some(letter_spacing_em) = style.letter_spacing_em {
                                    text = text.letter_spacing_em(letter_spacing_em);
                                }
                                pieces.push(text.into_element(cx));
                            }

                            pieces
                        },
                    )]
                },
                move |cx, row, col| vec![(cell_at)(cx, col, row.original)],
                output,
            )]
        })
    }
}
