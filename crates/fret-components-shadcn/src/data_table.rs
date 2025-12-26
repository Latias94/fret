use std::sync::Arc;

use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement};
use fret_core::geometry::Edges;
use fret_core::{Color, Px};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ColumnProps, GridProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::table::{TableCell, TableHead, TableRow};

#[derive(Debug, Clone)]
pub struct DataTableRowState {
    pub selected: bool,
    pub enabled: bool,
    pub on_click: Option<CommandId>,
}

impl Default for DataTableRowState {
    fn default() -> Self {
        Self {
            selected: false,
            enabled: true,
            on_click: None,
        }
    }
}

fn border_color(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .unwrap_or(theme.colors.panel_border)
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

/// shadcn/ui `DataTable` (virtualized table contract, first pass).
///
/// This is intentionally **not** TanStack Table parity. It is a stable, GPU-friendly contract for:
/// - a fixed header row
/// - a virtualized body with fixed row height
/// - shadcn-style row hover + selection affordances via `TableRow`
#[derive(Debug, Clone)]
pub struct DataTable {
    headers: Vec<Arc<str>>,
    rows: usize,
    overscan: usize,
    row_height: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl DataTable {
    pub fn new(headers: impl IntoIterator<Item = impl Into<Arc<str>>>, rows: usize) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows,
            overscan: 4,
            row_height: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
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

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        mut key_at: impl FnMut(usize) -> u64,
        mut row_state_at: impl FnMut(usize) -> DataTableRowState,
        mut cells_at: impl FnMut(&mut ElementCx<'_, H>, usize) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let cols = self.headers.len().max(1) as u16;
        let row_height = self.row_height.unwrap_or_else(|| row_height_px(&theme));
        let border = border_color(&theme);

        let root_chrome = ChromeRefinement::default()
            .rounded_md()
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(self.chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, self.layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        let headers = self.headers;
        let rows = self.rows;
        let overscan = self.overscan;

        cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = border_color(&theme);

            let header_row = {
                let header_chrome = ChromeRefinement::default()
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

                let header_cells: Vec<AnyElement> = headers
                    .iter()
                    .cloned()
                    .map(|h| TableHead::new(h).into_element(cx))
                    .collect();

                let header_theme = theme.clone();
                cx.container(props, move |cx| {
                    let mut grid = GridProps::default();
                    grid.cols = cols;
                    grid.gap = Px(0.0);
                    grid.padding = Edges::all(Px(0.0));
                    grid.justify = MainAlign::Start;
                    grid.layout = decl_style::layout_style(
                        &header_theme,
                        LayoutRefinement::default().w_full(),
                    );

                    let header_cells = header_cells.clone();
                    vec![cx.grid(grid, move |_cx| header_cells)]
                })
            };

            let body = cx.virtual_list_with_layout(
                list_layout_style(),
                rows,
                row_height,
                overscan,
                None,
                move |cx, range| {
                    range
                        .map(|i| {
                            let key = key_at(i);
                            cx.keyed(key, |cx| {
                                let state = row_state_at(i);
                                let is_last = i + 1 == rows;
                                let cells = cells_at(cx, i)
                                    .into_iter()
                                    .map(|c| TableCell::new(c).into_element(cx))
                                    .collect::<Vec<_>>();

                                TableRow::new(cols, cells)
                                    .selected(state.selected)
                                    .enabled(state.enabled)
                                    .border_bottom(!is_last)
                                    .on_click_opt(state.on_click)
                                    .into_element(cx)
                            })
                        })
                        .collect()
                },
            );

            let mut col = ColumnProps::default();
            col.layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
            col.gap = Px(0.0);
            col.padding = Edges::all(Px(0.0));
            col.align = fret_ui::element::CrossAlign::Stretch;
            col.justify = MainAlign::Start;

            vec![cx.column(col, move |_cx| vec![header_row, body])]
        })
    }
}

trait TableRowExt {
    fn on_click_opt(self, cmd: Option<CommandId>) -> Self;
}

impl TableRowExt for TableRow {
    fn on_click_opt(mut self, cmd: Option<CommandId>) -> Self {
        if let Some(cmd) = cmd {
            self = self.on_click(cmd);
        }
        self
    }
}
