//! Element-based data grid prototype.
//!
//! This surface exists for rich per-cell UI and interaction experiments.
//! For spreadsheet-scale density and a constant-ish UI node count, prefer the canvas-backed
//! `DataGridCanvas` (re-exported as `DataGrid` in `fret-ui-shadcn`).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::geometry::Edges;
use fret_core::{Color, Px};
use fret_runtime::CommandId;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, MainAlign, Overflow,
    PositionStyle, PressableProps, ScrollAxis, ScrollProps, ScrollbarAxis, ScrollbarProps,
    ScrollbarStyle, SemanticsProps, SizeStyle, StackProps, WheelRegionProps,
};
use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_headless::grid_viewport::{
    compute_grid_viewport_2d, default_range_extractor, GridAxisMetrics,
};
use fret_ui_kit::command::ElementCommandGatingExt as _;
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
struct DataGridViewportState {
    scroll: ScrollHandle,
    row_metrics: GridAxisMetrics<u64>,
    col_metrics: GridAxisMetrics<u64>,
    applied_col_signature: (u64, usize),
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
        row_state_at: impl FnMut(usize) -> DataGridRowState,
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

            let mut stack_layout = LayoutStyle::default();
            stack_layout.size.width = Length::Fill;
            stack_layout.size.height = Length::Fill;
            let cell_at = cell_at.clone();

            let stack = cx.stack_props(
                StackProps {
                    layout: stack_layout,
                },
                move |cx| {
                    let theme = Theme::global(&*cx.app).clone();
                    let theme_for_body = theme.clone();
                    let headers = headers.clone();
                    let col_widths = col_widths.clone();

                    let (scroll_handle, scroll_x, total_w, total_h, rows_visible, cols_visible) =
                        cx.with_state(DataGridViewportState::default, |state| {
                            let cols_signature = (cols_revision, cols);
                            state.row_metrics.ensure_fixed_with_key(
                                rows,
                                rows_revision,
                                row_height,
                                Px(0.0),
                                Px(0.0),
                                |i| i as u64,
                            );

                            if col_widths.is_empty() {
                                state.col_metrics.ensure_fixed_with_key(
                                    cols,
                                    cols_revision,
                                    Px(160.0),
                                    Px(0.0),
                                    Px(0.0),
                                    |i| i as u64,
                                );
                            } else {
                                state.col_metrics.ensure_measured_with_key(
                                    cols,
                                    cols_revision,
                                    Px(160.0),
                                    Px(0.0),
                                    Px(0.0),
                                    |i| i as u64,
                                );
                            }

                            if state.applied_col_signature != cols_signature {
                                if !col_widths.is_empty() {
                                    state.col_metrics.reset_measurements();
                                    for i in 0..cols {
                                        let w = col_widths.get(i).copied().unwrap_or(Px(160.0));
                                        state.col_metrics.measure(i, w);
                                    }
                                }
                                state.applied_col_signature = cols_signature;
                            }

                            let viewport = state.scroll.viewport_size();
                            let offset = state.scroll.offset();
                            let vp = compute_grid_viewport_2d(
                                &state.row_metrics,
                                &state.col_metrics,
                                offset.x,
                                offset.y,
                                viewport.width,
                                viewport.height,
                                overscan_rows,
                                overscan_cols,
                            );

                            let mut rows_visible = Vec::new();
                            let mut cols_visible = Vec::new();
                            let mut scroll_x = offset.x;
                            if let Some(vp) = vp {
                                scroll_x = vp.scroll_x;
                                rows_visible = default_range_extractor(vp.row_range)
                                    .into_iter()
                                    .filter_map(|i| state.row_metrics.axis_item(i))
                                    .collect();
                                cols_visible = default_range_extractor(vp.col_range)
                                    .into_iter()
                                    .filter_map(|i| state.col_metrics.axis_item(i))
                                    .collect();
                            }

                            (
                                state.scroll.clone(),
                                scroll_x,
                                state.col_metrics.total_size(),
                                state.row_metrics.total_size(),
                                rows_visible,
                                cols_visible,
                            )
                        });

                    let (body, body_id) = {
                        let mut body_layout = list_layout_style();
                        body_layout.overflow = Overflow::Clip;

                        let mut body_slot_layout = LayoutStyle::default();
                        body_slot_layout.position = PositionStyle::Absolute;
                        body_slot_layout.inset.top = Some(row_height);
                        body_slot_layout.inset.left = Some(Px(0.0));
                        body_slot_layout.inset.right = Some(Px(0.0));
                        body_slot_layout.inset.bottom = Some(Px(0.0));
                        body_slot_layout.size.width = Length::Fill;
                        body_slot_layout.size.height = Length::Fill;
                        body_slot_layout.overflow = Overflow::Clip;

                        let mut content_layout = LayoutStyle::default();
                        content_layout.size.width = Length::Px(total_w);
                        content_layout.size.height = Length::Px(total_h);
                        content_layout.overflow = Overflow::Visible;

                        let cell_at = cell_at.clone();
                        let cols_visible = cols_visible.clone();
                        let rows_visible = rows_visible.clone();

                        let body = cx.scroll(
                            ScrollProps {
                                layout: body_layout,
                                axis: ScrollAxis::Both,
                                scroll_handle: Some(scroll_handle.clone()),
                                ..Default::default()
                            },
                            move |cx| {
                                let cell_at = cell_at.clone();
                                let cols_visible = cols_visible.clone();

                                let mut row_key_at = row_key_at;
                                let mut row_state_at = row_state_at;

                                vec![cx.container(
                                    ContainerProps {
                                        layout: content_layout,
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        let mut out = Vec::new();
                                        for row in rows_visible.iter() {
                                            let st = row_state_at(row.index);
                                            let on_click = st.on_click;
                                            let selected = st.selected;
                                            let mut enabled = st.enabled;
                                            if let Some(cmd) = on_click.as_ref() {
                                                enabled = enabled && cx.command_is_enabled(cmd);
                                            }
                                            let row_key = row_key_at(row.index);
                                            let is_last = row.index + 1 == rows;

                                            let pressable_layout = LayoutStyle {
                                                position: PositionStyle::Absolute,
                                                inset: InsetStyle {
                                                    top: Some(row.start),
                                                    left: Some(Px(0.0)),
                                                    ..Default::default()
                                                },
                                                size: SizeStyle {
                                                    width: Length::Px(total_w),
                                                    height: Length::Px(row.size),
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            };
                                            let pressable = PressableProps {
                                                enabled,
                                                layout: pressable_layout,
                                                ..Default::default()
                                            };

                                            let row_theme = theme_for_body.clone();
                                            let row_cell_at = cell_at.clone();
                                            let row_cols_visible = cols_visible.clone();

                                            out.push(cx.keyed(row_key, move |cx| {
                                                cx.pressable(pressable, move |cx, state| {
                                                    cx.pressable_dispatch_command_if_enabled_opt(
                                                        on_click,
                                                    );

                                                    let mut hover_bg = row_bg;
                                                    hover_bg.a *= 0.5;
                                                    let selected_bg = row_bg;

                                                    let mut chrome = ChromeRefinement::default()
                                                        .border_1()
                                                        .border_color(ColorRef::Color(border));
                                                    if selected {
                                                        chrome =
                                                            chrome.bg(ColorRef::Color(selected_bg));
                                                    } else if state.hovered {
                                                        chrome =
                                                            chrome.bg(ColorRef::Color(hover_bg));
                                                    }

                                                    let mut row_container_layout =
                                                        LayoutStyle::default();
                                                    row_container_layout.size.width = Length::Fill;
                                                    row_container_layout.size.height = Length::Fill;
                                                    row_container_layout.overflow =
                                                        Overflow::Visible;

                                                    let mut props = decl_style::container_props(
                                                        &row_theme,
                                                        chrome,
                                                        LayoutRefinement::default().w_full(),
                                                    );
                                                    props.layout = row_container_layout;
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
                                                        let mut cells = Vec::new();
                                                        for col in row_cols_visible.iter() {
                                                            let cell_layout = LayoutStyle {
                                                                position: PositionStyle::Absolute,
                                                                inset: InsetStyle {
                                                                    top: Some(Px(0.0)),
                                                                    left: Some(col.start),
                                                                    ..Default::default()
                                                                },
                                                                size: SizeStyle {
                                                                    width: Length::Px(col.size),
                                                                    height: Length::Fill,
                                                                    ..Default::default()
                                                                },
                                                                ..Default::default()
                                                            };

                                                            let row_cell_at = row_cell_at.clone();
                                                            let row_index = row.index;
                                                            let col_index = col.index;
                                                            cells.push(cx.keyed(
                                                                (row_key, col.key),
                                                                move |cx| {
                                                                    let cell = row_cell_at
                                                                        .borrow_mut()(
                                                                        cx, row_index, col_index,
                                                                    );
                                                                    let cell = TableCell::new(cell)
                                                                        .into_element(cx);
                                                                    cx.container(
                                                                        ContainerProps {
                                                                            layout: cell_layout,
                                                                            ..Default::default()
                                                                        },
                                                                        move |_cx| vec![cell],
                                                                    )
                                                                },
                                                            ));
                                                        }
                                                        cells
                                                    })]
                                                })
                                            }));
                                        }
                                        out
                                    },
                                )]
                            },
                        );

                        let body_id = body.id;
                        let body = cx.semantics(
                            SemanticsProps {
                                layout: body_slot_layout,
                                test_id: Some(Arc::<str>::from("shadcn-data-grid-body")),
                                ..Default::default()
                            },
                            move |_cx| [body],
                        );

                        (body, body_id)
                    };

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
                        props.layout.overflow = Overflow::Clip;

                        let header_inner = cx.container(props, move |cx| {
                            let mut out = Vec::new();
                            for col in cols_visible.iter() {
                                let left = Px(col.start.0 - scroll_x.0);
                                let cell_layout = LayoutStyle {
                                    position: PositionStyle::Absolute,
                                    inset: InsetStyle {
                                        top: Some(Px(0.0)),
                                        left: Some(left),
                                        ..Default::default()
                                    },
                                    size: SizeStyle {
                                        width: Length::Px(col.size),
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                };

                                let text = headers.get(col.index).cloned().unwrap_or(Arc::from(""));
                                out.push(cx.keyed(col.key, move |cx| {
                                    let head = TableHead::new(text).into_element(cx);
                                    cx.container(
                                        ContainerProps {
                                            layout: cell_layout,
                                            ..Default::default()
                                        },
                                        move |_cx| vec![head],
                                    )
                                }));
                            }
                            out
                        });

                        cx.wheel_region(
                            WheelRegionProps {
                                axis: ScrollAxis::Both,
                                scroll_target: Some(body_id),
                                scroll_handle: scroll_handle.clone(),
                                ..Default::default()
                            },
                            move |_cx| vec![header_inner],
                        )
                    };
                    let header = cx.semantics(
                        SemanticsProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(row_height),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            test_id: Some(Arc::<str>::from("shadcn-data-grid-header")),
                            ..Default::default()
                        },
                        move |_cx| [header],
                    );

                    let show_scrollbar_x = scroll_handle.max_offset().x.0 > 0.01;
                    let show_scrollbar_y = scroll_handle.max_offset().y.0 > 0.01;

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
                            scroll_target: Some(body_id),
                            scroll_handle: scroll_handle.clone(),
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
                            scroll_target: Some(body_id),
                            scroll_handle: scroll_handle.clone(),
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
