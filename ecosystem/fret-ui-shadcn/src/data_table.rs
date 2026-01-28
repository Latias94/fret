use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Axis, Color, FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, VirtualListOptions,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::table::{
    TableRowMeasureMode, TableViewOutput, TableViewProps, table_virtualized,
};
use fret_ui_kit::declarative::{
    action_hooks::ActionHooksExt as _, model_watch::ModelWatchExt as _,
};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};

use fret_ui_headless::table::{
    ColumnDef, RowKey, SortSpec, TableState, sort_for_column, toggle_sort_for_column,
};

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
    /// - fixed-height rows only (measured rows are not supported yet)
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

        if measure_rows {
            return DataTable {
                overscan,
                row_height,
                measure_rows,
                chrome,
                layout,
                output: None,
            }
            .into_element(
                cx,
                data,
                data_revision,
                state,
                columns,
                get_row_key,
                header_label,
                cell_at,
            );
        }

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

            let row_h = row_height
                .or_else(|| theme.metric_by_key("component.table.row_height"))
                .unwrap_or(Px(28.0));

            let accent = theme
                .color_by_key("accent")
                .unwrap_or_else(|| theme.color_required("accent"));
            let row_hover_bg = Color {
                a: accent.a.min(0.10),
                ..accent
            };
            let row_selected_bg = Color {
                a: accent.a.min(0.16),
                ..accent
            };

            let get_row_key = Arc::new(get_row_key);
            let header_label = Arc::new(header_label);
            let cell_at = Arc::new(cell_at);

            #[derive(Debug, Clone, Copy)]
            struct RowEntry {
                key: RowKey,
                data_index: usize,
            }

            let state_value = cx.watch_model(&state).layout().cloned().unwrap_or_default();

            let mut entries: Vec<RowEntry> = (0..data.len())
                .map(|i| RowEntry {
                    key: (get_row_key)(&data[i], i, None),
                    data_index: i,
                })
                .collect();

            if let Some(SortSpec { column, desc }) = state_value.sorting.first() {
                if let Some(col) = columns.iter().find(|c| c.id.as_ref() == column.as_ref())
                    && let Some(cmp) = col.sort_cmp.as_ref()
                {
                    entries.sort_by(|a, b| {
                        let a = &data[a.data_index];
                        let b = &data[b.data_index];
                        let ord = (cmp)(a, b);
                        if *desc { ord.reverse() } else { ord }
                    });
                }
            }

            let entries: Arc<[RowEntry]> = Arc::from(entries);

            let mut list_layout = LayoutStyle::default();
            list_layout.size.width = Length::Fill;
            list_layout.size.height = Length::Fill;
            list_layout.flex.grow = 1.0;
            list_layout.flex.basis = Length::Px(Px(0.0));

            let mut list_options = VirtualListOptions::new(row_h, overscan);
            list_options.items_revision = items_revision;

            let header = {
                let state = state.clone();
                let columns = columns.clone();
                let header_label = Arc::clone(&header_label);
                let header_style = header_style.clone();
                let header_fg = header_fg;
                let sort_fg = sort_fg;

                let header_bg = theme
                    .color_by_key("muted")
                    .unwrap_or_else(|| theme.color_required("card"));

                cx.container(
                    ContainerProps {
                        background: Some(header_bg),
                        layout: LayoutStyle {
                            size: fret_ui::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(34.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        let sorting = cx
                            .watch_model(&state)
                            .paint()
                            .read_ref(|s| s.sorting.clone())
                            .ok()
                            .unwrap_or_default();

                        vec![cx.flex(
                            FlexProps {
                                layout: decl_style::layout_style(
                                    &Theme::global(&*cx.app),
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
                                columns
                                    .iter()
                                    .map(|col| {
                                        let label = (header_label)(col);
                                        let sort_state = sort_for_column(&sorting, col.id.as_ref());
                                        let col_id = col.id.clone();
                                        let state = state.clone();

                                        cx.pressable(
                                            PressableProps {
                                                enabled: true,
                                                a11y: PressableA11y {
                                                    role: Some(SemanticsRole::Button),
                                                    label: Some(label.clone()),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            move |cx, _st| {
                                                cx.pressable_update_model(&state, move |st| {
                                                    toggle_sort_for_column(
                                                        &mut st.sorting,
                                                        col_id.clone(),
                                                        false,
                                                    );
                                                    st.pagination.page_index = 0;
                                                });

                                                let mut pieces: Vec<AnyElement> = Vec::new();
                                                {
                                                    let mut text = ui::label(cx, label.clone())
                                                        .text_size_px(header_style.size)
                                                        .font_weight(header_style.weight)
                                                        .text_color(ColorRef::Color(header_fg))
                                                        .nowrap();
                                                    if let Some(line_height) =
                                                        header_style.line_height
                                                    {
                                                        text = text.line_height_px(line_height);
                                                    }
                                                    if let Some(letter_spacing_em) =
                                                        header_style.letter_spacing_em
                                                    {
                                                        text = text
                                                            .letter_spacing_em(letter_spacing_em);
                                                    }
                                                    pieces.push(text.into_element(cx));
                                                }

                                                if let Some(desc) = sort_state {
                                                    let indicator: Arc<str> = Arc::from(if desc {
                                                        " ▼"
                                                    } else {
                                                        " ▲"
                                                    });
                                                    let mut text = ui::label(cx, indicator)
                                                        .text_size_px(header_style.size)
                                                        .font_weight(header_style.weight)
                                                        .text_color(ColorRef::Color(sort_fg))
                                                        .nowrap();
                                                    if let Some(line_height) =
                                                        header_style.line_height
                                                    {
                                                        text = text.line_height_px(line_height);
                                                    }
                                                    if let Some(letter_spacing_em) =
                                                        header_style.letter_spacing_em
                                                    {
                                                        text = text
                                                            .letter_spacing_em(letter_spacing_em);
                                                    }
                                                    pieces.push(text.into_element(cx));
                                                }

                                                vec![cx.container(
                                                    ContainerProps {
                                                        padding: Edges::symmetric(
                                                            Px(12.0),
                                                            Px(0.0),
                                                        ),
                                                        ..Default::default()
                                                    },
                                                    move |_cx| pieces,
                                                )]
                                            },
                                        )
                                    })
                                    .collect::<Vec<_>>()
                            },
                        )]
                    },
                )
            };

            let key_at: Arc<dyn Fn(usize) -> fret_ui::ItemKey> = {
                let entries = entries.clone();
                Arc::new(move |i| entries[i].key.0)
            };

            let row = {
                let state = state.clone();
                let entries = entries.clone();
                let columns = columns.clone();
                let data = data.clone();
                let cell_at = Arc::clone(&cell_at);
                let debug_row_test_id_prefix = debug_row_test_id_prefix.clone();

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
                    let data_for_row = Arc::clone(&data);
                    let columns_for_row = Arc::clone(&columns);
                    let cell_at_for_row = Arc::clone(&cell_at);
                    cx.pressable(
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
                        move |cx, st| {
                            let state = state_model.clone();
                            cx.pressable_update_model(&state, move |st| {
                                if st.row_selection.contains(&row_key) {
                                    st.row_selection.remove(&row_key);
                                } else {
                                    st.row_selection.insert(row_key);
                                }
                            });

                            let bg = if selected {
                                Some(row_selected_bg)
                            } else if st.hovered {
                                Some(row_hover_bg)
                            } else {
                                None
                            };

                            let original = &data_for_row[data_index];
                            let cells = columns_for_row
                                .iter()
                                .map(|col| (cell_at_for_row)(cx, col, original))
                                .collect::<Vec<_>>();

                            vec![cx.container(
                                ContainerProps {
                                    background: bg,
                                    layout: LayoutStyle {
                                        size: fret_ui::element::SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Px(row_h),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    padding: Edges::symmetric(Px(12.0), Px(0.0)),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![cx.flex(
                                        FlexProps {
                                            layout: decl_style::layout_style(
                                                &Theme::global(&*cx.app),
                                                LayoutRefinement::default().w_full().h_px(row_h),
                                            ),
                                            direction: Axis::Horizontal,
                                            gap: Px(12.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: false,
                                        },
                                        move |_cx| cells,
                                    )]
                                },
                            )]
                        },
                    )
                })
            };

            vec![cx.flex(
                FlexProps {
                    layout: decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().h_full(),
                    ),
                    direction: Axis::Vertical,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |cx| {
                    vec![
                        header,
                        cx.virtual_list_keyed_retained_with_layout(
                            list_layout,
                            entries.len(),
                            list_options,
                            &scroll_handle,
                            key_at,
                            row,
                        ),
                    ]
                },
            )]
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
