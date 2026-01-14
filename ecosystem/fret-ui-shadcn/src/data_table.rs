use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ColumnProps, GridProps, LayoutStyle, Length, MainAlign, Overflow, PressableProps,
    ScrollAxis, WheelRegionProps,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius};

use fret_ui_kit::headless::table::{
    ColumnDef, RowKey, Table, TableState, is_row_selected, toggle_sort_for_column,
};

use crate::table::{TableCell, TableHead, TableRow};

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn row_height_px(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.table.row_min_h")
        .unwrap_or(Px(40.0))
}

fn list_layout_style() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));
    layout
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
/// Notes (v0):
/// - row activation toggles selection by `RowKey` (flat tables; sub-row selection is deferred)
/// - header activation toggles single-column sorting (multi-sort key modifiers are deferred)
#[derive(Debug, Clone)]
pub struct DataTable {
    overscan: usize,
    row_height: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Default for DataTable {
    fn default() -> Self {
        Self {
            overscan: 4,
            row_height: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost, TData>(
        self,
        cx: &mut ElementContext<'_, H>,
        data: Arc<[TData]>,
        data_revision: u64,
        state: Model<TableState>,
        columns: Vec<ColumnDef<TData>>,
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
            chrome,
            layout,
        } = self;

        let theme = Theme::global(&*cx.app).clone();
        let border = border_color(&theme);
        let row_height = row_height.unwrap_or_else(|| row_height_px(&theme));

        let state_value = cx
            .app
            .models()
            .read(&state, |v| v.clone())
            .unwrap_or_default();
        let state_revision = state.revision(&*cx.app).unwrap_or(0);
        let items_revision = mixed_revision(data_revision, state_revision);

        let table = Table::builder(data.as_ref())
            .columns(columns)
            .state(state_value)
            .get_row_key(get_row_key)
            .build();

        let visible_columns: Arc<[ColumnDef<TData>]> = table
            .visible_columns()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>()
            .into();
        let visible_headers: Arc<[Arc<str>]> = visible_columns
            .iter()
            .map(|c| header_label(c))
            .collect::<Vec<_>>()
            .into();

        let row_model = table.row_model().clone();
        let row_keys: Arc<[u64]> = row_model
            .flat_rows()
            .iter()
            .filter_map(|&i| row_model.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>()
            .into();
        let rows = row_keys.len();

        let root_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = border_color(&theme);

            let cols = visible_columns.len().max(1) as u16;
            let scroll_handle = cx.with_state(VirtualListScrollHandle::new, |h| h.clone());
            let mut options = fret_ui::element::VirtualListOptions::new(row_height, overscan);
            options.items_revision = items_revision;
            options.measure_mode = fret_ui::element::VirtualListMeasureMode::Fixed;
            options.key_cache = fret_ui::element::VirtualListKeyCacheMode::VisibleOnly;

            let state_for_rows = state.clone();
            let row_model_for_rows = row_model.clone();
            let cols_for_rows = visible_columns.clone();
            let cell_at = Arc::new(cell_at);
            let body = cx.virtual_list_keyed_with_layout(
                list_layout_style(),
                rows,
                options,
                &scroll_handle,
                {
                    let row_keys = row_keys.clone();
                    move |i| row_keys.get(i).copied().unwrap_or(i as u64)
                },
                move |cx, i| {
                    let key_u64 = row_keys.get(i).copied().unwrap_or(i as u64);
                    let row_key = RowKey(key_u64);

                    let selected = cx
                        .app
                        .models()
                        .read(&state_for_rows, |st| {
                            is_row_selected(row_key, &st.row_selection)
                        })
                        .unwrap_or(false);

                    let on_activate: OnActivate = {
                        let state = state_for_rows.clone();
                        Arc::new(move |host, _acx, _reason| {
                            let _ = host.models_mut().update(&state, |st| {
                                if st.row_selection.contains(&row_key) {
                                    st.row_selection.remove(&row_key);
                                } else {
                                    st.row_selection.insert(row_key);
                                }
                            });
                        })
                    };

                    let is_last = i + 1 == rows;
                    let row_index = row_model_for_rows.flat_rows().get(i).copied();
                    let row_data = row_index
                        .and_then(|ri| row_model_for_rows.row(ri))
                        .map(|r| r.original);

                    let cells = cols_for_rows
                        .iter()
                        .filter_map(|c| {
                            row_data.map(|d| TableCell::new(cell_at(cx, c, d)).into_element(cx))
                        })
                        .collect::<Vec<_>>();

                    TableRow::new(cols, cells)
                        .selected(selected)
                        .border_bottom(!is_last)
                        .on_activate(on_activate)
                        .into_element(cx)
                },
            );
            let body_id = body.id;

            let header_row = {
                let header_bg = theme
                    .color_by_key("muted")
                    .or_else(|| theme.color_by_key("muted.background"))
                    .unwrap_or_else(|| theme.color_required("muted.background"));
                let header_chrome = ChromeRefinement::default()
                    .bg(ColorRef::Color(header_bg))
                    .border_1()
                    .border_color(ColorRef::Color(border));
                let mut props = decl_style::container_props(
                    &theme,
                    header_chrome,
                    LayoutRefinement::default().w_full(),
                );
                props.border = Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: Px(1.0),
                    left: Px(0.0),
                };

                let state_for_header = state.clone();
                let header_cells: Vec<AnyElement> = visible_headers
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(idx, label)| {
                        let state_for_header = state_for_header.clone();
                        let column_id = visible_columns
                            .get(idx)
                            .map(|c| c.id.clone())
                            .unwrap_or_else(|| Arc::<str>::from(""));
                        cx.pressable(
                            PressableProps {
                                enabled: true,
                                layout: decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().w_full().h_full(),
                                ),
                                ..Default::default()
                            },
                            move |cx, _state| {
                                let state = state_for_header.clone();
                                let column_id = column_id.clone();
                                cx.pressable_on_activate(Arc::new(move |host, _acx, _reason| {
                                    let _ = host.models_mut().update(&state, |st| {
                                        toggle_sort_for_column(
                                            &mut st.sorting,
                                            column_id.clone(),
                                            false,
                                        );
                                    });
                                }));

                                vec![TableHead::new(label.clone()).into_element(cx)]
                            },
                        )
                    })
                    .collect();

                let header_theme = theme.clone();
                let header = cx.container(props, move |cx| {
                    let grid = GridProps {
                        cols,
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        layout: decl_style::layout_style(
                            &header_theme,
                            LayoutRefinement::default().w_full(),
                        ),
                        ..Default::default()
                    };

                    let header_cells = header_cells.clone();
                    vec![cx.grid(grid, move |_cx| header_cells)]
                });

                cx.wheel_region(
                    WheelRegionProps {
                        axis: ScrollAxis::Y,
                        scroll_target: Some(body_id),
                        scroll_handle: scroll_handle.base_handle().clone(),
                        ..Default::default()
                    },
                    move |_cx| vec![header],
                )
            };

            let col = ColumnProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                align: fret_ui::element::CrossAlign::Stretch,
                justify: MainAlign::Start,
            };

            vec![cx.column(col, move |_cx| vec![header_row, body])]
        })
    }
}

/// Backwards-compatible name for the previous `DataTableTanstack`.
///
/// Prefer `DataTable` for new code.
pub type DataTableTanstack = DataTable;
