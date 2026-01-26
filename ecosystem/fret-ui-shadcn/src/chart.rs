use std::sync::Arc;

use fret_core::{Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacerProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

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
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
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
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

        let layout = LayoutRefinement::default()
            .min_w(MetricRef::Px(min_w_content))
            .merge(self.layout);

        let props = decl_style::container_props(&theme, chrome, layout);

        let mut children = Vec::new();
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

        let row_gap = decl_style::space(&theme, Space::N2);
        let dot = Px(10.0);
        let line_w = Px(4.0);

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
                ChartTooltipIndicator::Line | ChartTooltipIndicator::Dashed => CrossAlign::Stretch,
            };

            children.push(cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
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
                stack::VStackProps::default().gap(Space::N1p5).items_start(),
                move |_cx| children,
            );

            vec![ui::stack(cx, move |_cx| vec![sentinel, body]).into_element(cx)]
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
        let legend_gap = decl_style::space(&theme, Space::N4);

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
                        layout: LayoutStyle::default(),
                        direction: fret_core::Axis::Horizontal,
                        gap: item_gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
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
                    wrap: false,
                },
                move |_cx| items,
            )]
        })
    }
}
