use std::collections::BTreeMap;
use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_icons::IconId;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsDecoration, SizeStyle, SpacerProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

/// Upstream shadcn/ui v4 `ChartConfig` entry.
#[derive(Debug, Clone)]
pub struct ChartConfigItem {
    pub label: Option<Arc<str>>,
    pub icon: Option<IconId>,
    pub color: Option<ColorRef>,
}

impl ChartConfigItem {
    pub fn new() -> Self {
        Self {
            label: None,
            icon: None,
            color: None,
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for ChartConfigItem {
    fn default() -> Self {
        Self::new()
    }
}

/// Upstream shadcn/ui v4 `ChartConfig` (a key -> entry map).
pub type ChartConfig = BTreeMap<Arc<str>, ChartConfigItem>;

/// Chart context surface aligned with upstream `useChart`.
#[derive(Debug, Clone)]
pub struct ChartContext {
    pub chart_id: Arc<str>,
    pub config: Arc<ChartConfig>,
}

pub fn chart_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<ChartContext> {
    cx.provided::<ChartContext>().cloned()
}

#[track_caller]
pub fn use_chart<H: UiHost>(cx: &ElementContext<'_, H>) -> ChartContext {
    chart_context(cx).expect("use_chart must be used within a `ChartContainer`")
}

/// shadcn/ui v4 `ChartTooltip`.
///
/// Upstream exports the Recharts `Tooltip` primitive. Fret does not yet wire chart engine tooltips,
/// so this is a thin wrapper that exists to preserve the part surface shape. Today it simply
/// renders the configured [`ChartTooltipContent`].
#[derive(Debug, Clone)]
pub struct ChartTooltip {
    content: ChartTooltipContent,
}

impl ChartTooltip {
    pub fn new(content: ChartTooltipContent) -> Self {
        Self { content }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.content.into_element(cx)
    }
}

/// shadcn/ui v4 `ChartLegend`.
///
/// Upstream exports the Recharts `Legend` primitive. Fret does not yet wire chart engine legends,
/// so this is a thin wrapper that exists to preserve the part surface shape. Today it simply
/// renders the configured [`ChartLegendContent`].
#[derive(Debug, Clone)]
pub struct ChartLegend {
    content: ChartLegendContent,
}

impl ChartLegend {
    pub fn new(content: ChartLegendContent) -> Self {
        Self { content }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.content.into_element(cx)
    }
}

/// shadcn/ui v4 `ChartStyle`.
///
/// Upstream uses `<style>` injection to define `--color-*` CSS variables under a chart-scoped
/// selector. Fret does not have CSS variables, so this is currently a no-op element that exists to
/// keep the part surface aligned.
#[derive(Debug, Clone)]
pub struct ChartStyle {
    id: Arc<str>,
    config: Arc<ChartConfig>,
}

impl ChartStyle {
    pub fn new(id: impl Into<Arc<str>>, config: Arc<ChartConfig>) -> Self {
        Self {
            id: id.into(),
            config,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let _ = (self.id, self.config);
        cx.container(
            ContainerProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(0.0)),
                        height: Length::Px(Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }
}

/// shadcn/ui v4 `ChartContainer`.
///
/// Note: Upstream owns Recharts wiring and CSS variable injection. In Fret, `ChartContainer` is a
/// lightweight context + layout wrapper. The actual chart engine integration lives elsewhere.
#[derive(Debug, Clone)]
pub struct ChartContainer {
    id: Option<Arc<str>>,
    config: Arc<ChartConfig>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

impl ChartContainer {
    pub fn new(config: ChartConfig) -> Self {
        Self {
            id: None,
            config: Arc::new(config),
            // Upstream uses `aspect-video` and centers the chart surface.
            layout: LayoutRefinement::default()
                .w_full()
                .aspect_ratio(16.0 / 9.0),
            test_id: None,
        }
    }

    pub fn id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        child: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let layout = decl_style::layout_style(&theme, self.layout);
        let config = self.config.clone();
        let chart_id: Arc<str> = self
            .id
            .clone()
            .map(|id| Arc::<str>::from(format!("chart-{id}")))
            .unwrap_or_else(|| Arc::<str>::from("chart"));
        let context = ChartContext {
            chart_id: chart_id.clone(),
            config: config.clone(),
        };

        cx.provide(context, |cx| {
            let mut el = cx.container(
                ContainerProps {
                    layout,
                    ..Default::default()
                },
                move |cx| {
                    vec![
                        ChartStyle::new(chart_id.clone(), config.clone()).into_element(cx),
                        child(cx),
                    ]
                },
            );

            if let Some(test_id) = self.test_id {
                el = el.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Panel)
                        .label("chart")
                        .test_id(test_id),
                );
            } else {
                el = el.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Panel)
                        .label("chart"),
                );
            }

            el
        })
    }
}

fn wrap_panel_semantics<H: UiHost>(
    _cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    child: AnyElement,
) -> AnyElement {
    child.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Panel)
            .label(label),
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::ElementKind;
    use std::sync::Mutex;

    #[test]
    fn chart_panel_semantics_stamps_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let child = cx.text("X");
            wrap_panel_semantics(cx, Arc::from("Panel"), child)
        });

        assert!(
            !matches!(element.kind, ElementKind::Semantics(_)),
            "expected chart panel wrappers to avoid `Semantics` wrappers; use `attach_semantics` instead"
        );
        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Panel)
        );
        assert_eq!(
            element
                .semantics_decoration
                .as_ref()
                .and_then(|d| d.label.as_deref()),
            Some("Panel")
        );
    }

    #[test]
    fn chart_container_installs_context_for_use_chart() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let captured: Arc<Mutex<Option<ChartContext>>> = Arc::new(Mutex::new(None));
        let captured_for_child = captured.clone();

        let mut config = ChartConfig::default();
        config.insert(Arc::from("sales"), ChartConfigItem::new().label("Sales"));

        let _ = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config)
                .id("a")
                .test_id("chart-container")
                .into_element(cx, move |cx| {
                    let ctx = use_chart(cx);
                    *captured_for_child.lock().expect("lock") = Some(ctx);
                    cx.text("chart")
                })
        });

        let ctx = captured.lock().expect("lock").clone().expect("context");
        assert_eq!(ctx.chart_id.as_ref(), "chart-a");
        assert!(
            ctx.config.contains_key("sales"),
            "expected chart config to be visible through `use_chart`"
        );
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let text_xs_px = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_PX)
            .unwrap_or(Px(12.0));
        let text_xs_line_height = theme
            .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
            .unwrap_or(Px(16.0));

        let bg = theme.color_token("background");
        let border = theme
            .color_by_key("border/50")
            .or_else(|| theme.color_by_key("border"))
            .unwrap_or_else(|| theme.color_token("border"));

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
                            ui::text(label)
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
                            .unwrap_or_else(|| theme.color_token("foreground"));

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
                        ui::text(item.label)
                            .text_xs()
                            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                            .line_height_px(text_xs_px)
                            .h_px(MetricRef::Px(text_xs_px))
                            .into_element(cx),
                    );
                    row.push(cx.spacer(SpacerProps::default()));
                    row.push(
                        ui::text(item.value)
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
                            gap: row_gap.into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                    let label = ui::text(item.label)
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let value = ui::text(item.value)
                        .text_xs()
                        .font_medium()
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let suffix = ui::text("kcal")
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
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
                            gap: Px(2.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                        .unwrap_or_else(|| theme.color_token("foreground"));
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
                    let label = ui::text(item.label)
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let value = ui::text(item.value)
                        .text_xs()
                        .font_medium()
                        .line_height_px(text_xs_line_height)
                        .h_px(MetricRef::Px(text_xs_line_height))
                        .into_element(cx);
                    let suffix = ui::text("kcal")
                        .text_xs()
                        .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
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
                            gap: row_gap.into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                        let total_label = ui::text("Total")
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_line_height)
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx);
                        let total_value = ui::text(total.clone())
                            .text_xs()
                            .font_medium()
                            .line_height_px(text_xs_line_height)
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx);
                        let total_suffix = ui::text("kcal")
                            .text_xs()
                            .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
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
                                }
                                .into(),
                                border: Edges {
                                    top: Px(1.0),
                                    right: Px(0.0),
                                    bottom: Px(0.0),
                                    left: Px(0.0),
                                },
                                border_color: Some(theme.color_token("border")),
                                ..Default::default()
                            },
                            move |cx| {
                                vec![cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: fret_core::Axis::Horizontal,
                                        gap: Px(2.0).into(),
                                        padding: Edges::all(Px(0.0)).into(),
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

                        let group = ui::v_stack(move |_cx| vec![line, total_row])
                            .gap(Space::N3p5)
                            .items_stretch()
                            .layout(LayoutRefinement::default().w_full())
                            .into_element(cx);

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

            let body = ui::v_stack(move |_cx| children)
                .gap(Space::N1p5)
                .items_stretch()
                .layout(LayoutRefinement::default().w_full())
                .into_element(cx);

            vec![
                ui::stack(move |_cx| vec![sentinel, body])
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
            gap: Space::N4.into(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
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
                    .unwrap_or_else(|| theme.color_token("foreground"));

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

                let label = ui::text(item.label)
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
                        gap: item_gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
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
                    gap: legend_gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: self.wrap,
                },
                move |_cx| items,
            )]
        })
    }
}
