use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Color, Px};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, MainAlign, Overflow,
    PositionStyle, PressableProps, ScrollAxis, ScrollbarAxis, ScrollbarProps, ScrollbarStyle,
    SizeStyle, StackProps, WheelRegionProps,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement};

use crate::table::{TableCell, TableHead};

#[derive(Debug, Clone)]
pub struct DataGridRowState {
    pub selected: bool,
    pub enabled: bool,
    pub on_click: Option<CommandId>,
}

impl Default for DataGridRowState {
    fn default() -> Self {
        Self {
            selected: false,
            enabled: true,
            on_click: None,
        }
    }
}

#[derive(Debug, Default, Clone)]
struct DataGridScrollHandles {
    rows: VirtualListScrollHandle,
    cols: VirtualListScrollHandle,
}

fn with_alpha(mut c: Color, a: f32) -> Color {
    c.a = a.clamp(0.0, 1.0);
    c
}

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn muted_bg(theme: &Theme) -> Color {
    theme.color_required("muted")
}

fn row_height_px(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.table.row_min_h")
        .unwrap_or(Px(40.0))
}

fn scrollbar_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("metric.scrollbar.width")
        .unwrap_or(Px(10.0))
}

fn scrollbar_thumb(theme: &Theme) -> Color {
    theme
        .color_by_key("scrollbar.thumb.background")
        .unwrap_or_else(|| with_alpha(theme.color_required("muted-foreground"), 0.35))
}

fn scrollbar_thumb_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("scrollbar.thumb.hover.background")
        .unwrap_or_else(|| with_alpha(theme.color_required("muted-foreground"), 0.55))
}

fn list_layout_style() -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Fill;
    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = Length::Px(Px(0.0));
    layout
}

fn fixed_width_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    width: Px,
    child: AnyElement,
) -> AnyElement {
    let mut layout = LayoutStyle::default();
    layout.size.width = Length::Px(width);
    layout.size.height = Length::Fill;
    cx.container(
        ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

#[derive(Debug, Clone)]
pub struct DataGrid {
    headers: Vec<Arc<str>>,
    rows: usize,
    overscan_rows: usize,
    overscan_cols: usize,
    row_height: Option<Px>,
    col_widths: Option<Vec<Px>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl DataGrid {
    pub fn new(headers: impl IntoIterator<Item = impl Into<Arc<str>>>, rows: usize) -> Self {
        Self {
            headers: headers.into_iter().map(Into::into).collect(),
            rows,
            overscan_rows: 4,
            overscan_cols: 2,
            row_height: None,
            col_widths: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn overscan_rows(mut self, overscan: usize) -> Self {
        self.overscan_rows = overscan;
        self
    }

    pub fn overscan_cols(mut self, overscan: usize) -> Self {
        self.overscan_cols = overscan;
        self
    }

    pub fn row_height(mut self, row_height: Px) -> Self {
        self.row_height = Some(row_height);
        self
    }

    pub fn col_widths(mut self, widths: impl IntoIterator<Item = Px>) -> Self {
        self.col_widths = Some(widths.into_iter().collect());
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

    #[allow(clippy::too_many_arguments)]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        rows_revision: u64,
        cols_revision: u64,
        row_key_at: impl FnMut(usize) -> u64,
        mut row_state_at: impl FnMut(usize) -> DataGridRowState,
        cell_at: impl FnMut(&mut ElementContext<'_, H>, usize, usize) -> AnyElement,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let cols = self.headers.len().max(1);
        let row_height = self.row_height.unwrap_or_else(|| row_height_px(&theme));
        let border = border_color(&theme);
        let scrollbar_w = scrollbar_width(&theme);
        let thumb = scrollbar_thumb(&theme);
        let thumb_hover = scrollbar_thumb_hover(&theme);

        let root_chrome = ChromeRefinement::default()
            .rounded_md()
            .border_1()
            .border_color(ColorRef::Color(border))
            .merge(self.chrome);
        let mut root_props = decl_style::container_props(&theme, root_chrome, self.layout.w_full());
        root_props.layout.overflow = Overflow::Clip;

        let headers: Arc<[Arc<str>]> = Arc::from(self.headers);
        let col_widths: Arc<[Px]> = Arc::from(self.col_widths.unwrap_or_default());
        let cell_at = Rc::new(RefCell::new(cell_at));

        let rows = self.rows;
        let overscan_rows = self.overscan_rows;
        let overscan_cols = self.overscan_cols;

        cx.container(root_props, move |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let border = border_color(&theme);
            let row_bg = muted_bg(&theme);

            let handles = cx.with_state(DataGridScrollHandles::default, |h| h.clone());
            let row_handle = handles.rows.clone();
            let col_handle = handles.cols.clone();

            let mut stack_layout = LayoutStyle::default();
            stack_layout.size.width = Length::Fill;
            stack_layout.size.height = Length::Fill;

            let mut row_key_at = row_key_at;
            let cell_at = cell_at.clone();

            let stack = cx.stack_props(
                StackProps {
                    layout: stack_layout,
                },
                move |cx| {
                    let theme = Theme::global(&*cx.app).clone();
                    let col_widths = col_widths.clone();

                    let header = {
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
                        props.layout.size.height = Length::Px(row_height);

                        let headers = headers.clone();
                        let col_handle = col_handle.clone();
                        let header_col_widths = col_widths.clone();
                        let header_inner = cx.container(props, move |cx| {
                            let mut options =
                                fret_ui::element::VirtualListOptions::new(Px(160.0), overscan_cols);
                            options.axis = fret_core::Axis::Horizontal;
                            options.items_revision = cols_revision;

                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;

                            vec![cx.virtual_list_keyed_with_layout(
                                layout,
                                cols,
                                options,
                                &col_handle,
                                |i| i as u64,
                                move |cx, col| {
                                    let text = headers.get(col).cloned().unwrap_or(Arc::from(""));
                                    let width =
                                        header_col_widths.get(col).copied().unwrap_or(Px(160.0));
                                    let head = TableHead::new(text).into_element(cx);
                                    fixed_width_container(cx, width, head)
                                },
                            )]
                        });

                        cx.wheel_region(
                            WheelRegionProps {
                                axis: ScrollAxis::Y,
                                scroll_target: None,
                                scroll_handle: row_handle.base_handle().clone(),
                                ..Default::default()
                            },
                            move |_cx| vec![header_inner],
                        )
                    };

                    let body = {
                        let mut options =
                            fret_ui::element::VirtualListOptions::new(row_height, overscan_rows);
                        options.axis = fret_core::Axis::Vertical;
                        options.items_revision = rows_revision;

                        let col_handle = col_handle.clone();
                        let theme = theme.clone();
                        let row_handle = row_handle.clone();

                        cx.virtual_list_keyed_with_layout(
                            list_layout_style(),
                            rows,
                            options,
                            &row_handle,
                            &mut row_key_at,
                            move |cx, row| {
                                let st = row_state_at(row);
                                let is_last = row + 1 == rows;

                                let pressable_layout = {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height = Length::Px(row_height);
                                    layout
                                };
                                let pressable = PressableProps {
                                    enabled: st.enabled,
                                    layout: pressable_layout,
                                    ..Default::default()
                                };

                                let on_click = st.on_click;
                                let selected = st.selected;

                                let col_handle = col_handle.clone();
                                let col_widths = col_widths.clone();
                                let cell_at = cell_at.clone();
                                let theme_for_row = theme.clone();

                                cx.pressable(pressable, move |cx, state| {
                                    cx.pressable_dispatch_command_opt(on_click);
                                    let theme = Theme::global(&*cx.app).clone();

                                    let mut hover_bg = row_bg;
                                    hover_bg.a *= 0.5;
                                    let selected_bg = row_bg;

                                    let border = border_color(&theme);
                                    let mut chrome = ChromeRefinement::default()
                                        .border_1()
                                        .border_color(ColorRef::Color(border));
                                    if selected {
                                        chrome = chrome.bg(ColorRef::Color(selected_bg));
                                    } else if state.hovered {
                                        chrome = chrome.bg(ColorRef::Color(hover_bg));
                                    }

                                    let layout = LayoutRefinement::default().w_full();
                                    let mut props =
                                        decl_style::container_props(&theme_for_row, chrome, layout);
                                    props.layout.size.height = Length::Px(row_height);
                                    props.layout.overflow = Overflow::Visible;
                                    props.border = if !is_last {
                                        Edges {
                                            top: Px(0.0),
                                            right: Px(0.0),
                                            bottom: Px(1.0),
                                            left: Px(0.0),
                                        }
                                    } else {
                                        Edges::all(Px(0.0))
                                    };

                                    vec![cx.container(props, move |cx| {
                                        let mut options = fret_ui::element::VirtualListOptions::new(
                                            Px(160.0),
                                            overscan_cols,
                                        );
                                        options.axis = fret_core::Axis::Horizontal;
                                        options.items_revision = cols_revision;

                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Fill;
                                        layout.size.height = Length::Fill;

                                        let row_cell_at = cell_at.clone();
                                        vec![cx.virtual_list_keyed_with_layout(
                                            layout,
                                            cols,
                                            options,
                                            &col_handle,
                                            |i| i as u64,
                                            move |cx, col| {
                                                let cell = row_cell_at.borrow_mut()(cx, row, col);
                                                let width = col_widths
                                                    .get(col)
                                                    .copied()
                                                    .unwrap_or(Px(160.0));
                                                let cell = TableCell::new(cell).into_element(cx);
                                                fixed_width_container(cx, width, cell)
                                            },
                                        )]
                                    })]
                                })
                            },
                        )
                    };

                    let show_scrollbar_x = col_handle.base_handle().max_offset().x.0 > 0.01;
                    let show_scrollbar_y = row_handle.base_handle().max_offset().y.0 > 0.01;

                    let mut out = vec![header, body];

                    if show_scrollbar_y {
                        let scrollbar_layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                top: Some(row_height),
                                right: Some(Px(0.0)),
                                bottom: Some(if show_scrollbar_x {
                                    scrollbar_w
                                } else {
                                    Px(0.0)
                                }),
                                left: None,
                            },
                            size: SizeStyle {
                                width: Length::Px(scrollbar_w),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        out.push(cx.scrollbar(ScrollbarProps {
                            layout: scrollbar_layout,
                            axis: ScrollbarAxis::Vertical,
                            scroll_target: None,
                            scroll_handle: row_handle.base_handle().clone(),
                            style: ScrollbarStyle {
                                thumb,
                                thumb_hover,
                                ..Default::default()
                            },
                        }));
                    }

                    if show_scrollbar_x {
                        let scrollbar_layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                top: None,
                                right: Some(if show_scrollbar_y {
                                    scrollbar_w
                                } else {
                                    Px(0.0)
                                }),
                                bottom: Some(Px(0.0)),
                                left: Some(Px(0.0)),
                            },
                            size: SizeStyle {
                                height: Length::Px(scrollbar_w),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        out.push(cx.scrollbar(ScrollbarProps {
                            layout: scrollbar_layout,
                            axis: ScrollbarAxis::Horizontal,
                            scroll_target: None,
                            scroll_handle: col_handle.base_handle().clone(),
                            style: ScrollbarStyle {
                                thumb,
                                thumb_hover,
                                ..Default::default()
                            },
                        }));
                    }

                    if show_scrollbar_x && show_scrollbar_y {
                        let corner_layout = LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                right: Some(Px(0.0)),
                                bottom: Some(Px(0.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(scrollbar_w),
                                height: Length::Px(scrollbar_w),
                                ..Default::default()
                            },
                            ..Default::default()
                        };

                        out.push(cx.container(
                            ContainerProps {
                                layout: corner_layout,
                                background: Some(Color::TRANSPARENT),
                                ..Default::default()
                            },
                            |_cx| vec![],
                        ));
                    }

                    out
                },
            );

            let col = ColumnProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                align: fret_ui::element::CrossAlign::Stretch,
                justify: MainAlign::Start,
            };

            vec![cx.column(col, move |_cx| vec![stack])]
        })
    }
}
