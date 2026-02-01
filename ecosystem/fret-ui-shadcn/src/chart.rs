use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsProps, SizeStyle, SpacerProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

fn wrap_panel_semantics<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    child: AnyElement,
) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Panel,
            label: Some(label),
            ..Default::default()
        },
        move |_cx| vec![child],
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTooltipIndicator {
    Dot,
    Line,
    Dashed,
}

impl Default for ChartTooltipIndicator {
    fn default() -> Self {
        Self::Dot
    }
}

#[derive(Debug, Clone)]
pub struct ChartTooltipItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub color: Option<ColorRef>,
}

impl ChartTooltipItem {
    pub fn new(label: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartTooltipContentKind {
    Default,
    FormatterKcal,
    AdvancedKcalTotal,
}

impl Default for ChartTooltipContentKind {
    fn default() -> Self {
        Self::Default
    }
}

/// shadcn/ui v4 chart tooltip content.
///
/// Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/chart.tsx` (`ChartTooltipContent`).
#[derive(Debug, Clone)]
pub struct ChartTooltipContent {
    label: Option<Arc<str>>,
    items: Vec<ChartTooltipItem>,
    indicator: ChartTooltipIndicator,
    hide_label: bool,
    hide_indicator: bool,
    kind: ChartTooltipContentKind,
    fixed_width_border_box: Option<Px>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    test_id_prefix: Option<Arc<str>>,
}

impl Default for ChartTooltipContent {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartTooltipContent {
    pub fn new() -> Self {
        Self {
            label: None,
            items: Vec::new(),
            indicator: ChartTooltipIndicator::Dot,
            hide_label: false,
            hide_indicator: false,
            kind: ChartTooltipContentKind::Default,
            fixed_width_border_box: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            test_id_prefix: None,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ChartTooltipItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn indicator(mut self, indicator: ChartTooltipIndicator) -> Self {
        self.indicator = indicator;
        self
    }

    pub fn hide_label(mut self, hide: bool) -> Self {
        self.hide_label = hide;
        self
    }

    pub fn hide_indicator(mut self, hide: bool) -> Self {
        self.hide_indicator = hide;
        self
    }

    pub fn kind(mut self, kind: ChartTooltipContentKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn fixed_width_border_box(mut self, width: Px) -> Self {
        self.fixed_width_border_box = Some(width);
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

    pub fn test_id_prefix(mut self, prefix: impl Into<Arc<str>>) -> Self {
        self.test_id_prefix = Some(prefix.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text_xs_px = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_PX)
            .unwrap_or(Px(12.0));
        let text_xs_line_height = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
            .unwrap_or(Px(16.0));

        let bg = theme.color_required("background");
        let border = theme
            .color_by_key("border/50")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or_else(|| theme.color_required("border"));

        let test_id_prefix = self.test_id_prefix.clone();

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .px(Space::N2p5)
            .py(Space::N1p5)
            .shadow_xl()
            .merge(self.chrome);

        // shadcn `ChartTooltipContent` uses `min-w-[8rem]` under `box-border`. Fret's `Container`
        // padding/border live outside of `LayoutStyle.size`, so we convert the border-box minimum
        // into a content-box minimum.
        let min_w_border_box = Px(128.0);
        let padding_x = decl_style::space(&theme, Space::N2p5);
        let border_w = Px(1.0);
        let min_w_content =
            Px((min_w_border_box.0 - padding_x.0 * 2.0 - border_w.0 * 2.0).max(0.0));

        let mut layout = LayoutRefinement::default().min_w(MetricRef::Px(min_w_content));
        if let Some(border_box_width) = self.fixed_width_border_box {
            layout = layout.w_px(MetricRef::Px(border_box_width));
        }
        layout = layout.merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);

        let mut children = Vec::new();

        let row_gap = decl_style::space(&theme, Space::N2);
        let dot = Px(10.0);
        let line_w = Px(4.0);

        match self.kind {
            ChartTooltipContentKind::Default => {
                if !self.hide_label {
                    if let Some(label) = self.label.clone() {
                        children.push(
                            ui::text(cx, label)
                                .text_xs()
                                .font_medium()
                                .h_px(MetricRef::Px(text_xs_line_height))
                                .into_element(cx),
                        );
                    }
                }

                for item in self.items {
                    let mut row = Vec::new();

                    if !self.hide_indicator {
                        let indicator_color = item
                            .color
                            .as_ref()
                            .map(|c| c.resolve(&theme))
                            .unwrap_or_else(|| theme.color_required("foreground"));

                        let (w, h) = match self.indicator {
                            ChartTooltipIndicator::Dot => (dot, dot),
                            ChartTooltipIndicator::Line | ChartTooltipIndicator::Dashed => {
                                (line_w, Px(0.0))
                            }
                        };

                        let mut indicator_props = decl_style::container_props(
                            &theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(indicator_color))
                                .border_1()
                                .border_color(ColorRef::Color(indicator_color)),
                            LayoutRefinement::default(),
                        );
                        indicator_props.corner_radii = Corners::all(Px(2.0));
                        indicator_props.layout.size.width = fret_ui::element::Length::Px(w);
                        if h.0 > 0.0 {
                            indicator_props.layout.size.height = fret_ui::element::Length::Px(h);
                        }

                        row.push(cx.container(indicator_props, |_cx| Vec::new()));
                    }

                    row.push(
                        ui::text(cx, item.label)
                            .text_xs()
                            .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                            .line_height_px(text_xs_px)
                            .h_px(MetricRef::Px(text_xs_px))
                            .into_element(cx),
                    );
                    row.push(cx.spacer(SpacerProps::default()));
                    row.push(
                        ui::text(cx, item.value)
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_px)
                            .h_px(MetricRef::Px(text_xs_px))
                            .into_element(cx),
                    );

                    let align = match self.indicator {
                        ChartTooltipIndicator::Dot => CrossAlign::Center,
                        ChartTooltipIndicator::Line | ChartTooltipIndicator::Dashed => {
                            CrossAlign::Stretch
                        }
                    };

                    children.push(cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout
                            },
                            direction: fret_core::Axis::Horizontal,
                            gap: row_gap,
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align,
                            wrap: false,
                        },
                        move |_cx| row,
                    ));
                }
            }
            ChartTooltipContentKind::FormatterKcal => {
                let row_min_w = Px(130.0);
                for item in self.items {
                    let label = ui::text(cx, item.label)
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let value = ui::text(cx, item.value)
                        .text_xs()
                        .font_medium()
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let suffix = ui::text(cx, "kcal")
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);

                    children.push(cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(row_min_w);
                                layout.size.height = Length::Px(text_xs_line_height);
                                layout
                            },
                            direction: fret_core::Axis::Horizontal,
                            gap: Px(2.0),
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| vec![label, cx.spacer(SpacerProps::default()), value, suffix],
                    ));
                }
            }
            ChartTooltipContentKind::AdvancedKcalTotal => {
                let indicator_square = Px(10.0);

                let total = self.items.iter().fold(0.0_f32, |acc, item| {
                    acc + item.value.parse::<f32>().unwrap_or(0.0)
                });
                let total = Arc::<str>::from(format!("{total:.0}"));

                for (index, item) in self.items.into_iter().enumerate() {
                    let indicator_color = item
                        .color
                        .as_ref()
                        .map(|c| c.resolve(&theme))
                        .unwrap_or_else(|| theme.color_required("foreground"));
                    let indicator = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(indicator_square),
                                    height: Length::Px(indicator_square),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            background: Some(indicator_color),
                            corner_radii: Corners::all(Px(2.0)),
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    );
                    let label = ui::text(cx, item.label)
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let value = ui::text(cx, item.value)
                        .text_xs()
                        .font_medium()
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let suffix = ui::text(cx, "kcal")
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);

                    let line = cx.flex(
                        FlexProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout
                            },
                            direction: fret_core::Axis::Horizontal,
                            gap: row_gap,
                            padding: Edges::all(Px(0.0)),
                            justify: MainAlign::Start,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |cx| {
                            vec![
                                indicator,
                                label,
                                cx.spacer(SpacerProps::default()),
                                value,
                                suffix,
                            ]
                        },
                    );

                    let line = if index == 1 {
                        line
                    } else if let Some(prefix) = test_id_prefix.as_ref() {
                        wrap_panel_semantics(cx, Arc::from(format!("{prefix}:item-{index}")), line)
                    } else {
                        line
                    };

                    if index == 1 {
                        let total_label = ui::text(cx, "Total")
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_line_height)
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx);
                        let total_value = ui::text(cx, total.clone())
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_line_height)
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx);
                        let total_suffix = ui::text(cx, "kcal")
                            .text_xs()
                            .text_color(ColorRef::Color(theme.color_required("muted-foreground")))
                            .line_height_px(text_xs_line_height)
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx);

                        let total_row = cx.container(
                            ContainerProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Fill;
                                    layout.size.height =
                                        Length::Px(Px(text_xs_line_height.0 + 7.0));
                                    layout
                                },
                                padding: Edges {
                                    top: Px(6.0),
                                    right: Px(0.0),
                                    bottom: Px(0.0),
                                    left: Px(0.0),
                                },
                                border: Edges {
                                    top: Px(1.0),
                                    right: Px(0.0),
                                    bottom: Px(0.0),
                                    left: Px(0.0),
                                },
                                border_color: Some(theme.color_required("border")),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(2.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        vec![
                                            total_label,
                                            cx.spacer(SpacerProps::default()),
                                            total_value,
                                            total_suffix,
                                        ]
                                    },
                                )]
                            },
                        );

                        let total_row = if let Some(prefix) = test_id_prefix.as_ref() {
                            wrap_panel_semantics(
                                cx,
                                Arc::from(format!("{prefix}:total-row")),
                                total_row,
                            )
                        } else {
                            total_row
                        };

                        let group = stack::vstack(
                            cx,
                            stack::VStackProps::default()
                                .gap(Space::N3p5)
                                .items_stretch()
                                .layout(LayoutRefinement::default().w_full()),
                            move |_cx| vec![line, total_row],
                        );

                        let group = if let Some(prefix) = test_id_prefix.as_ref() {
                            wrap_panel_semantics(
                                cx,
                                Arc::from(format!("{prefix}:item-{index}")),
                                group,
                            )
                        } else {
                            group
                        };

                        children.push(group);
                    } else {
                        children.push(line);
                    }
                }
            }
        }

        cx.container(props, move |cx| {
            // `min-w-[8rem]` is implemented via `LayoutStyle.size.min_width`, but Fret's min-size
            // constraints are not consistently enforced across all containers yet.
            //
            // Add an invisible sentinel node so auto-sized tooltips still match web geometry while
            // allowing expansion when content is wider than 8rem.
            let sentinel = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(min_w_content),
                            height: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );

            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1p5)
                    .items_stretch()
                    .layout(LayoutRefinement::default().w_full()),
                move |_cx| children,
            );

            vec![
                ui::stack(cx, move |_cx| vec![sentinel, body])
                    .w_full()
                    .into_element(cx),
            ]
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartLegendVerticalAlign {
    Top,
    Bottom,
}

impl Default for ChartLegendVerticalAlign {
    fn default() -> Self {
        Self::Bottom
    }
}

#[derive(Debug, Clone)]
pub struct ChartLegendItem {
    pub label: Arc<str>,
    pub color: Option<ColorRef>,
}

impl ChartLegendItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            color: None,
        }
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }
}

/// shadcn/ui v4 chart legend content.
///
/// Upstream: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/chart.tsx` (`ChartLegendContent`).
#[derive(Debug, Clone)]
pub struct ChartLegendContent {
    items: Vec<ChartLegendItem>,
    vertical_align: ChartLegendVerticalAlign,
    hide_icon: bool,
    wrap: bool,
    gap: Space,
    item_width_px: Option<Px>,
    item_justify_center: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Default for ChartLegendContent {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartLegendContent {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            vertical_align: ChartLegendVerticalAlign::Bottom,
            hide_icon: false,
            wrap: false,
            gap: Space::N4,
            item_width_px: None,
            item_justify_center: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ChartLegendItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn vertical_align(mut self, vertical_align: ChartLegendVerticalAlign) -> Self {
        self.vertical_align = vertical_align;
        self
    }

    pub fn hide_icon(mut self, hide: bool) -> Self {
        self.hide_icon = hide;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn item_width_px(mut self, width: Px) -> Self {
        self.item_width_px = Some(width);
        self
    }

    pub fn item_justify_center(mut self, center: bool) -> Self {
        self.item_justify_center = center;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text_xs_line_height = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
            .unwrap_or(Px(16.0));

        let mut chrome = match self.vertical_align {
            ChartLegendVerticalAlign::Top => ChromeRefinement::default().pb(Space::N3),
            ChartLegendVerticalAlign::Bottom => ChromeRefinement::default().pt(Space::N3),
        };
        chrome = chrome.merge(self.chrome);

        let layout = LayoutRefinement::default().w_full().merge(self.layout);
        let outer_props = decl_style::container_props(&theme, chrome, layout);

        let item_gap = decl_style::space(&theme, Space::N1p5);
        let legend_gap = decl_style::space(&theme, self.gap);

        let items = self
            .items
            .into_iter()
            .map(|item| {
                let color = item
                    .color
                    .as_ref()
                    .map(|c| c.resolve(&theme))
                    .unwrap_or_else(|| theme.color_required("foreground"));

                let indicator = cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(8.0)),
                                height: Length::Px(Px(8.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        background: Some(color),
                        corner_radii: Corners::all(Px(2.0)),
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                );

                let label = ui::text(cx, item.label)
                    .text_xs()
                    .line_height_px(text_xs_line_height)
                    .h_px(MetricRef::Px(text_xs_line_height))
                    .into_element(cx);

                let mut row = Vec::new();
                row.push(indicator);
                row.push(label);

                cx.flex(
                    FlexProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            if let Some(width) = self.item_width_px {
                                layout.size.width = Length::Px(width);
                            }
                            layout
                        },
                        direction: fret_core::Axis::Horizontal,
                        gap: item_gap,
                        padding: Edges::all(Px(0.0)),
                        justify: if self.item_justify_center {
                            MainAlign::Center
                        } else {
                            MainAlign::Start
                        },
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |_cx| row,
                )
            })
            .collect::<Vec<_>>();

        cx.container(outer_props, move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: legend_gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: self.wrap,
                },
                move |_cx| items,
            )]
        })
    }
}
