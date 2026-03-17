use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use fret_chart::{ChartCanvasOutput, TooltipTextLine, TooltipTextLineKind};
use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_icons::IconId;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsDecoration, SizeStyle, SpacerProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
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
    pub output_model: Option<Model<ChartCanvasOutput>>,
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
/// Upstream exports the Recharts `Tooltip` primitive. Fret keeps the recipe surface lightweight and
/// optional: callers can still render fully static tooltip content, or opt into auto-derived
/// payloads by sharing a `ChartCanvasOutput` model through `ChartContainer::output_model(...)`.
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
    output_model: Option<Model<ChartCanvasOutput>>,
    layout: LayoutRefinement,
    test_id: Option<Arc<str>>,
}

pub struct ChartContainerBuild<H, B> {
    container: ChartContainer,
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> ChartContainerBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    pub fn id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.container = self.container.id(id);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.container = self.container.refine_layout(layout);
        self
    }

    pub fn output_model(mut self, output: Model<ChartCanvasOutput>) -> Self {
        self.container = self.container.output_model(output);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.container = self.container.test_id(id);
        self
    }

    #[track_caller]
    pub fn into_element(mut self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let build = self
            .build
            .take()
            .expect("chart container builder already consumed");
        self.container.into_element(cx, build)
    }
}

pub fn chart_container<H: UiHost, F>(config: ChartConfig, f: F) -> ChartContainerBuild<H, F>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    ChartContainerBuild {
        container: ChartContainer::new(config),
        build: Some(f),
        _phantom: PhantomData,
    }
}

impl ChartContainer {
    pub fn new(config: ChartConfig) -> Self {
        Self {
            id: None,
            config: Arc::new(config),
            output_model: None,
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

    pub fn output_model(mut self, output: Model<ChartCanvasOutput>) -> Self {
        self.output_model = Some(output);
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
            output_model: self.output_model.clone(),
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
    use fret_ui::element::{ElementKind, Length, TextProps};
    use std::sync::Mutex;

    fn find_text<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(props),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text(child, needle)),
        }
    }

    fn find_flex_with_texts<'a>(
        element: &'a AnyElement,
        needles: &[&str],
    ) -> Option<&'a AnyElement> {
        match &element.kind {
            ElementKind::Flex(_)
                if needles.iter().all(|needle| {
                    element.children.iter().any(|child| {
                        matches!(
                            &child.kind,
                            ElementKind::Text(props) if props.text.as_ref() == *needle
                        )
                    })
                }) =>
            {
                Some(element)
            }
            _ => element
                .children
                .iter()
                .find_map(|child| find_flex_with_texts(child, needles)),
        }
    }

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

    #[test]
    fn chart_container_builder_installs_context_for_use_chart() {
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
            chart_container(config, move |cx| {
                let ctx = use_chart(cx);
                *captured_for_child.lock().expect("lock") = Some(ctx);
                cx.text("chart")
            })
            .id("builder")
            .test_id("chart-container-builder")
            .into_element(cx)
        });

        let ctx = captured.lock().expect("lock").clone().expect("context");
        assert_eq!(ctx.chart_id.as_ref(), "chart-builder");
        assert!(
            ctx.config.contains_key("sales"),
            "expected chart config to be visible through `chart_container` builder"
        );
    }

    #[test]
    fn chart_legend_uses_chart_config_items_when_items_are_omitted() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let mut config = ChartConfig::default();
        config.insert(
            Arc::from("desktop"),
            ChartConfigItem::new()
                .label("Desktop")
                .icon(IconId::new_static("lucide.monitor")),
        );
        config.insert(Arc::from("mobile"), ChartConfigItem::new().label("Mobile"));

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config)
                .into_element(cx, |cx| ChartLegendContent::new().into_element(cx))
        });

        assert!(
            find_text(&element, "Desktop").is_some(),
            "expected legend fallback to render config label `Desktop`"
        );
        assert!(
            find_text(&element, "Mobile").is_some(),
            "expected legend fallback to render config label `Mobile`"
        );
    }

    #[test]
    fn chart_tooltip_uses_output_model_when_label_and_items_are_omitted() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let output = app.models_mut().insert(ChartCanvasOutput::default());
        let _ = output.update(&mut app, |state, _cx| {
            state.snapshot.tooltip_lines = vec![
                TooltipTextLine::columns("x (Month)", "January")
                    .with_kind(TooltipTextLineKind::AxisHeader),
                TooltipTextLine::columns("Desktop", "186")
                    .with_kind(TooltipTextLineKind::SeriesRow),
                TooltipTextLine::columns("Mobile", "80").with_kind(TooltipTextLineKind::SeriesRow),
            ];
        });

        let mut config = ChartConfig::default();
        config.insert(
            Arc::from("desktop"),
            ChartConfigItem::new()
                .label("Desktop")
                .icon(IconId::new_static("lucide.monitor")),
        );
        config.insert(
            Arc::from("mobile"),
            ChartConfigItem::new()
                .label("Mobile")
                .icon(IconId::new_static("lucide.smartphone")),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config)
                .output_model(output.clone())
                .into_element(cx, |cx| ChartTooltipContent::new().into_element(cx))
        });

        assert!(
            find_text(&element, "January").is_some(),
            "expected tooltip to derive the label from the output model"
        );
        assert!(
            find_text(&element, "Desktop").is_some(),
            "expected tooltip to derive the first series label from the output model"
        );
        assert!(
            find_text(&element, "186").is_some(),
            "expected tooltip to derive the first series value from the output model"
        );
        assert!(
            find_text(&element, "Mobile").is_some(),
            "expected tooltip to derive the second series label from the output model"
        );
        assert!(
            find_text(&element, "80").is_some(),
            "expected tooltip to derive the second series value from the output model"
        );
    }

    #[test]
    fn chart_tooltip_label_formatter_transforms_auto_label_and_normalizes_auto_items() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let output = app.models_mut().insert(ChartCanvasOutput::default());
        let _ = output.update(&mut app, |state, _cx| {
            state.snapshot.tooltip_lines = vec![
                TooltipTextLine::columns("x (Month)", "2024-07-16")
                    .with_kind(TooltipTextLineKind::AxisHeader),
                TooltipTextLine::columns("desktop", "186")
                    .with_kind(TooltipTextLineKind::SeriesRow),
                TooltipTextLine::columns("mobile", "80").with_kind(TooltipTextLineKind::SeriesRow),
            ];
        });

        let mut config = ChartConfig::default();
        config.insert(
            Arc::from("desktop"),
            ChartConfigItem::new().label("Desktop"),
        );
        config.insert(Arc::from("mobile"), ChartConfigItem::new().label("Mobile"));

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config)
                .output_model(output.clone())
                .into_element(cx, |cx| {
                    ChartTooltipContent::new()
                        .label_formatter(|context| {
                            format!(
                                "{}: {}",
                                context.key.as_deref().unwrap_or_default(),
                                context.label.as_deref().unwrap_or_default()
                            )
                        })
                        .into_element(cx)
                })
        });

        assert!(
            find_text(&element, "x (Month): 2024-07-16").is_some(),
            "expected label formatter to receive the derived axis label and key"
        );
        assert!(
            find_text(&element, "Desktop").is_some(),
            "expected auto-derived items to prefer ChartConfig labels"
        );
        assert!(
            find_text(&element, "Mobile").is_some(),
            "expected auto-derived items to keep ChartConfig label normalization"
        );
        assert!(
            find_text(&element, "desktop").is_none(),
            "expected raw series keys to stay out of the rendered tooltip labels"
        );
    }

    #[test]
    fn chart_tooltip_formatter_customizes_item_suffix_and_row_width() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartTooltipContent::new()
                .hide_label(true)
                .items([ChartTooltipItem::new("Running", "380")
                    .key("running")
                    .meta("unit", "kcal")])
                .formatter(|context| {
                    let series = context.item.key.as_deref().unwrap_or_default();
                    let unit = context
                        .item
                        .metadata
                        .get("unit")
                        .cloned()
                        .unwrap_or_default();
                    ChartTooltipFormattedItem::from_item(&context.item)
                        .label(format!("Series {series}"))
                        .value_suffix(unit)
                        .row_min_width(Px(130.0))
                })
                .into_element(cx)
        });

        assert!(
            find_text(&element, "Series running").is_some(),
            "expected formatter to customize the row label from item metadata"
        );
        assert!(
            find_text(&element, "380").is_some(),
            "expected formatter to preserve the item value"
        );
        assert!(
            find_text(&element, "kcal").is_some(),
            "expected formatter to append a custom suffix"
        );

        let row = find_flex_with_texts(&element, &["Series running", "380", "kcal"])
            .expect("expected formatter row flex element");
        let ElementKind::Flex(props) = &row.kind else {
            panic!("expected formatter row to be a flex element");
        };
        assert_eq!(
            props.layout.size.width,
            Length::Px(Px(130.0)),
            "expected formatter to control row minimum geometry"
        );
    }

    #[test]
    fn chart_tooltip_label_key_uses_chart_config_entry() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let mut config = ChartConfig::default();
        config.insert(
            Arc::from("visitors"),
            ChartConfigItem::new().label("Total Visitors"),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config).into_element(cx, |cx| {
                ChartTooltipContent::new()
                    .label("187")
                    .label_key("visitors")
                    .items([ChartTooltipItem::new("Chrome", "187")])
                    .into_element(cx)
            })
        });

        assert!(
            find_text(&element, "Total Visitors").is_some(),
            "expected label_key to resolve the tooltip header from ChartConfig"
        );
        assert!(
            find_text(&element, "Chrome").is_some(),
            "expected tooltip rows to stay visible after label_key remapping"
        );
    }

    #[test]
    fn chart_tooltip_name_key_uses_item_metadata_to_resolve_config() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(200.0)),
        );

        let mut config = ChartConfig::default();
        config.insert(Arc::from("chrome"), ChartConfigItem::new().label("Chrome"));
        config.insert(Arc::from("safari"), ChartConfigItem::new().label("Safari"));

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            ChartContainer::new(config).into_element(cx, |cx| {
                ChartTooltipContent::new()
                    .hide_label(true)
                    .name_key("browser")
                    .items([
                        ChartTooltipItem::new("ignored", "187").meta("browser", "chrome"),
                        ChartTooltipItem::new("ignored", "200").meta("browser", "safari"),
                    ])
                    .into_element(cx)
            })
        });

        assert!(
            find_text(&element, "Chrome").is_some(),
            "expected name_key metadata lookup to resolve the first config label"
        );
        assert!(
            find_text(&element, "Safari").is_some(),
            "expected name_key metadata lookup to resolve the second config label"
        );
        assert!(
            find_text(&element, "ignored").is_none(),
            "expected metadata-backed config labels to replace the raw item labels"
        );
    }
}

fn chart_config_entry_for_label<'a>(
    config: &'a ChartConfig,
    label: &str,
) -> Option<(Arc<str>, &'a ChartConfigItem)> {
    config
        .get_key_value(label)
        .map(|(key, item)| (key.clone(), item))
        .or_else(|| {
            config.iter().find_map(|(key, item)| {
                let matches_key = key.as_ref().eq_ignore_ascii_case(label);
                let matches_label = item
                    .label
                    .as_deref()
                    .is_some_and(|item_label| item_label.eq_ignore_ascii_case(label));
                if matches_key || matches_label {
                    Some((key.clone(), item))
                } else {
                    None
                }
            })
        })
}

fn chart_config_entry_for_key<'a>(
    config: &'a ChartConfig,
    key: &str,
) -> Option<(Arc<str>, &'a ChartConfigItem)> {
    config
        .get_key_value(key)
        .map(|(key, item)| (key.clone(), item))
}

fn chart_config_label_for_key(config: &ChartConfig, key: &str) -> Option<Arc<str>> {
    chart_config_entry_for_key(config, key)
        .map(|(config_key, item)| item.label.clone().unwrap_or(config_key))
}

#[derive(Debug, Clone, Default)]
struct ChartTooltipDerivedLabel {
    label: Option<Arc<str>>,
    key: Option<Arc<str>>,
    metadata: BTreeMap<Arc<str>, Arc<str>>,
}

fn chart_tooltip_auto_label(lines: &[TooltipTextLine]) -> ChartTooltipDerivedLabel {
    lines
        .iter()
        .find_map(|line| {
            if line.kind != TooltipTextLineKind::AxisHeader {
                return None;
            }

            if let Some((left, right)) = line.columns.as_ref() {
                return Some(ChartTooltipDerivedLabel {
                    label: Some(Arc::<str>::from(right.as_str())),
                    key: Some(Arc::<str>::from(left.as_str())),
                    metadata: BTreeMap::default(),
                });
            }

            Some(ChartTooltipDerivedLabel {
                label: Some(Arc::<str>::from(line.text.clone())),
                key: None,
                metadata: BTreeMap::default(),
            })
        })
        .unwrap_or_default()
}

fn chart_tooltip_auto_items(
    lines: &[TooltipTextLine],
    config: &ChartConfig,
) -> Vec<ChartTooltipItem> {
    lines
        .iter()
        .filter(|line| line.kind == TooltipTextLineKind::SeriesRow)
        .map(|line| {
            let (label, value) = match line.columns.as_ref() {
                Some((left, right)) => (left.as_str(), right.as_str()),
                None => (line.text.as_str(), ""),
            };

            let mut item = ChartTooltipItem::new(label, value);
            if let Some((config_key, config_item)) = chart_config_entry_for_label(config, label) {
                item = item
                    .key(config_key.clone())
                    .meta("config_key", config_key.clone())
                    .meta("source_label", label);
                if let Some(config_label) = config_item.label.clone() {
                    item = item.label(config_label);
                }
                if let Some(color) = config_item.color.clone() {
                    item = item.color(color);
                }
                if let Some(icon) = config_item.icon.clone() {
                    item = item.icon(icon);
                }
            }
            item
        })
        .collect()
}

fn chart_tooltip_resolve_label_key(
    label: Option<Arc<str>>,
    key: Option<Arc<str>>,
    metadata: &BTreeMap<Arc<str>, Arc<str>>,
    label_key: &str,
    config: Option<&ChartConfig>,
) -> Option<Arc<str>> {
    if label_key == "key" {
        let Some(resolved_key) = key else {
            return label;
        };
        return Some(
            config
                .and_then(|config| chart_config_label_for_key(config, &resolved_key))
                .unwrap_or(resolved_key),
        );
    }

    if let Some(resolved_key) = metadata.get(label_key).cloned() {
        return Some(
            config
                .and_then(|config| chart_config_label_for_key(config, &resolved_key))
                .unwrap_or(resolved_key),
        );
    }

    config
        .and_then(|config| chart_config_label_for_key(config, label_key))
        .or(label)
}

fn chart_tooltip_apply_name_key(
    mut item: ChartTooltipItem,
    name_key: &str,
    config: Option<&ChartConfig>,
) -> ChartTooltipItem {
    let resolved_key = if name_key == "key" {
        item.key.clone()
    } else {
        item.metadata.get(name_key).cloned()
    };

    let Some(resolved_key) = resolved_key else {
        return item;
    };

    item = item
        .key(resolved_key.clone())
        .meta("config_key", resolved_key.clone());

    if let Some(config) = config
        && let Some((config_key, config_item)) = chart_config_entry_for_key(config, &resolved_key)
    {
        item = item.label(config_item.label.clone().unwrap_or(config_key));
        if item.color.is_none()
            && let Some(color) = config_item.color.clone()
        {
            item = item.color(color);
        }
        if item.icon.is_none()
            && let Some(icon) = config_item.icon.clone()
        {
            item = item.icon(icon);
        }
        return item;
    }

    item.label(resolved_key)
}

#[derive(Debug, Clone)]
pub struct ChartTooltipItem {
    pub label: Arc<str>,
    pub value: Arc<str>,
    pub color: Option<ColorRef>,
    pub icon: Option<IconId>,
    pub key: Option<Arc<str>>,
    pub metadata: BTreeMap<Arc<str>, Arc<str>>,
}

impl ChartTooltipItem {
    pub fn new(label: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
            icon: None,
            key: None,
            metadata: BTreeMap::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = label.into();
        self
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = value.into();
        self
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn key(mut self, key: impl Into<Arc<str>>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn meta(mut self, key: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChartTooltipLabelContext {
    pub label: Option<Arc<str>>,
    pub items: Vec<ChartTooltipItem>,
    pub key: Option<Arc<str>>,
    pub metadata: BTreeMap<Arc<str>, Arc<str>>,
}

#[derive(Debug, Clone)]
pub struct ChartTooltipItemFormatContext {
    pub item: ChartTooltipItem,
    pub index: usize,
    pub label: Option<Arc<str>>,
}

#[derive(Debug, Clone, Default)]
pub struct ChartTooltipFormattedItem {
    label: Option<Arc<str>>,
    value: Option<Arc<str>>,
    value_suffix: Option<Arc<str>>,
    row_min_width: Option<Px>,
}

impl ChartTooltipFormattedItem {
    pub fn new(label: impl Into<Arc<str>>, value: impl Into<Arc<str>>) -> Self {
        Self {
            label: Some(label.into()),
            value: Some(value.into()),
            value_suffix: None,
            row_min_width: None,
        }
    }

    pub fn from_item(item: &ChartTooltipItem) -> Self {
        Self::new(item.label.clone(), item.value.clone())
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn clear_label(mut self) -> Self {
        self.label = None;
        self
    }

    pub fn value(mut self, value: impl Into<Arc<str>>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn clear_value(mut self) -> Self {
        self.value = None;
        self
    }

    pub fn value_suffix(mut self, value_suffix: impl Into<Arc<str>>) -> Self {
        self.value_suffix = Some(value_suffix.into());
        self
    }

    pub fn clear_value_suffix(mut self) -> Self {
        self.value_suffix = None;
        self
    }

    pub fn row_min_width(mut self, width: Px) -> Self {
        self.row_min_width = Some(width);
        self
    }
}

type ChartTooltipLabelFormatterFn =
    Arc<dyn Fn(&ChartTooltipLabelContext) -> Arc<str> + Send + Sync + 'static>;

#[derive(Clone)]
struct ChartTooltipLabelFormatter {
    f: ChartTooltipLabelFormatterFn,
}

impl std::fmt::Debug for ChartTooltipLabelFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChartTooltipLabelFormatter").finish()
    }
}

impl ChartTooltipLabelFormatter {
    fn new<F, R>(f: F) -> Self
    where
        F: Fn(&ChartTooltipLabelContext) -> R + Send + Sync + 'static,
        R: Into<Arc<str>> + 'static,
    {
        Self {
            f: Arc::new(move |cx| f(cx).into()),
        }
    }

    fn format(&self, cx: &ChartTooltipLabelContext) -> Arc<str> {
        (self.f)(cx)
    }
}

type ChartTooltipItemFormatterFn = Arc<
    dyn Fn(&ChartTooltipItemFormatContext) -> ChartTooltipFormattedItem + Send + Sync + 'static,
>;

#[derive(Clone)]
struct ChartTooltipItemFormatter {
    f: ChartTooltipItemFormatterFn,
}

impl std::fmt::Debug for ChartTooltipItemFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ChartTooltipItemFormatter").finish()
    }
}

impl ChartTooltipItemFormatter {
    fn new<F>(f: F) -> Self
    where
        F: Fn(&ChartTooltipItemFormatContext) -> ChartTooltipFormattedItem + Send + Sync + 'static,
    {
        Self { f: Arc::new(f) }
    }

    fn format(&self, cx: &ChartTooltipItemFormatContext) -> ChartTooltipFormattedItem {
        (self.f)(cx)
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
    name_key: Option<Arc<str>>,
    label_key: Option<Arc<str>>,
    label_formatter: Option<ChartTooltipLabelFormatter>,
    formatter: Option<ChartTooltipItemFormatter>,
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
            name_key: None,
            label_key: None,
            label_formatter: None,
            formatter: None,
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

    pub fn name_key(mut self, name_key: impl Into<Arc<str>>) -> Self {
        self.name_key = Some(name_key.into());
        self
    }

    pub fn label_key(mut self, label_key: impl Into<Arc<str>>) -> Self {
        self.label_key = Some(label_key.into());
        self
    }

    pub fn label_formatter<F, R>(mut self, formatter: F) -> Self
    where
        F: Fn(&ChartTooltipLabelContext) -> R + Send + Sync + 'static,
        R: Into<Arc<str>> + 'static,
    {
        self.label_formatter = Some(ChartTooltipLabelFormatter::new(formatter));
        self
    }

    pub fn formatter<F>(mut self, formatter: F) -> Self
    where
        F: Fn(&ChartTooltipItemFormatContext) -> ChartTooltipFormattedItem + Send + Sync + 'static,
    {
        self.formatter = Some(ChartTooltipItemFormatter::new(formatter));
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
        let Self {
            label,
            items,
            indicator,
            hide_label,
            hide_indicator,
            name_key,
            label_key: label_key_selector,
            label_formatter,
            formatter,
            kind,
            fixed_width_border_box,
            chrome,
            layout: layout_refinement,
            test_id_prefix,
        } = self;

        let chart_ctx = chart_context(cx);
        let mut label = label;
        let mut items = items;
        let mut derived_label_key = None;
        let mut label_metadata = BTreeMap::default();
        let mut bound_output_model = false;

        if (label.is_none() || items.is_empty())
            && let Some(chart_ctx) = chart_ctx.as_ref()
            && let Some(output_model) = chart_ctx.output_model.as_ref()
        {
            bound_output_model = true;
            cx.observe_model(output_model, Invalidation::Layout);
            if let Ok(output) = output_model.read(cx.app, |_app, output| output.clone()) {
                if label.is_none() {
                    let derived_label = chart_tooltip_auto_label(&output.snapshot.tooltip_lines);
                    label = derived_label.label;
                    derived_label_key = derived_label.key;
                    label_metadata = derived_label.metadata;
                }
                if items.is_empty() {
                    items =
                        chart_tooltip_auto_items(&output.snapshot.tooltip_lines, &chart_ctx.config);
                }
            }
        }

        if bound_output_model && items.is_empty() {
            return cx.container(ContainerProps::default(), |_cx| Vec::new());
        }

        let chart_config = chart_ctx.as_ref().map(|ctx| ctx.config.as_ref());
        if let Some(name_key) = name_key.as_deref() {
            items = items
                .into_iter()
                .map(|item| chart_tooltip_apply_name_key(item, name_key, chart_config))
                .collect();
        }
        if let Some(label_key) = label_key_selector.as_deref() {
            label = chart_tooltip_resolve_label_key(
                label,
                derived_label_key.clone(),
                &label_metadata,
                label_key,
                chart_config,
            );
        }

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

        let chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .bg(ColorRef::Color(bg))
            .border_1()
            .border_color(ColorRef::Color(border))
            .px(Space::N2p5)
            .py(Space::N1p5)
            .shadow_xl()
            .merge(chrome);

        // shadcn `ChartTooltipContent` uses `min-w-[8rem]` under `box-border`. Fret's `Container`
        // padding/border live outside of `LayoutStyle.size`, so we convert the border-box minimum
        // into a content-box minimum.
        let min_w_border_box = Px(128.0);
        let padding_x = decl_style::space(&theme, Space::N2p5);
        let border_w = Px(1.0);
        let min_w_content =
            Px((min_w_border_box.0 - padding_x.0 * 2.0 - border_w.0 * 2.0).max(0.0));

        let mut layout = LayoutRefinement::default().min_w(MetricRef::Px(min_w_content));
        if let Some(border_box_width) = fixed_width_border_box {
            layout = layout.w_px(MetricRef::Px(border_box_width));
        }
        layout = layout.merge(layout_refinement);

        let props = decl_style::container_props(&theme, chrome, layout);

        let mut children = Vec::new();

        let row_gap = decl_style::space(&theme, Space::N2);
        let dot = Px(10.0);
        let line_w = Px(4.0);
        let rendered_label = if hide_label {
            None
        } else if let Some(label_formatter) = label_formatter.as_ref() {
            Some(label_formatter.format(&ChartTooltipLabelContext {
                label: label.clone(),
                items: items.clone(),
                key: derived_label_key.clone(),
                metadata: label_metadata.clone(),
            }))
        } else {
            label.clone()
        };

        match kind {
            ChartTooltipContentKind::Default => {
                if let Some(label) = rendered_label.clone() {
                    children.push(
                        ui::text(label)
                            .text_xs()
                            .font_medium()
                            .h_px(MetricRef::Px(text_xs_line_height))
                            .into_element(cx),
                    );
                }

                for (index, item) in items.into_iter().enumerate() {
                    if let Some(formatter) = formatter.as_ref() {
                        let formatted = formatter.format(&ChartTooltipItemFormatContext {
                            item: item.clone(),
                            index,
                            label: rendered_label.clone().or_else(|| label.clone()),
                        });
                        let label = formatted.label.map(|label| {
                            ui::text(label)
                                .text_xs()
                                .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                                .line_height_px(text_xs_line_height)
                                .h_px(MetricRef::Px(text_xs_line_height))
                                .into_element(cx)
                        });
                        let value = formatted.value.map(|value| {
                            ui::text(value)
                                .text_xs()
                                .font_medium()
                                .line_height_px(text_xs_line_height)
                                .h_px(MetricRef::Px(text_xs_line_height))
                                .into_element(cx)
                        });
                        let suffix = formatted.value_suffix.map(|value_suffix| {
                            ui::text(value_suffix)
                                .text_xs()
                                .text_color(ColorRef::Color(theme.color_token("muted-foreground")))
                                .line_height_px(text_xs_line_height)
                                .h_px(MetricRef::Px(text_xs_line_height))
                                .into_element(cx)
                        });

                        children.push(cx.flex(
                            FlexProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    if let Some(row_min_width) = formatted.row_min_width {
                                        layout.size.width = Length::Px(row_min_width);
                                    }
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
                            move |cx| {
                                let mut row = Vec::new();
                                if let Some(label) = label {
                                    row.push(label);
                                }
                                if value.is_some() {
                                    if !row.is_empty() {
                                        row.push(cx.spacer(SpacerProps::default()));
                                    }
                                    row.push(value.expect("value presence already checked"));
                                }
                                if let Some(suffix) = suffix {
                                    row.push(suffix);
                                }
                                row
                            },
                        ));
                        continue;
                    }

                    let mut row = Vec::new();
                    let has_icon = item.icon.is_some();

                    if let Some(icon) = item.icon {
                        row.push(decl_icon::icon_with(cx, icon, Some(Px(10.0)), None));
                    } else if !hide_indicator {
                        let indicator_color = item
                            .color
                            .as_ref()
                            .map(|c| c.resolve(&theme))
                            .unwrap_or_else(|| theme.color_token("foreground"));

                        let (w, h) = match indicator {
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

                    let align = match indicator {
                        ChartTooltipIndicator::Dot if !has_icon => CrossAlign::Center,
                        ChartTooltipIndicator::Dot => CrossAlign::Center,
                        ChartTooltipIndicator::Line | ChartTooltipIndicator::Dashed => {
                            if has_icon {
                                CrossAlign::Center
                            } else {
                                CrossAlign::Stretch
                            }
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
                for item in items {
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

                let total = items.iter().fold(0.0_f32, |acc, item| {
                    acc + item.value.parse::<f32>().unwrap_or(0.0)
                });
                let total = Arc::<str>::from(format!("{total:.0}"));

                for (index, item) in items.into_iter().enumerate() {
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

        let mut element = cx.container(props, move |cx| {
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
        });

        if let Some(prefix) = test_id_prefix {
            element = element.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Panel)
                    .label(prefix.clone())
                    .test_id(prefix),
            );
        }

        element
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
    pub icon: Option<IconId>,
}

impl ChartLegendItem {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            color: None,
            icon: None,
        }
    }

    pub fn color(mut self, color: ColorRef) -> Self {
        self.color = Some(color);
        self
    }

    pub fn icon(mut self, icon: IconId) -> Self {
        self.icon = Some(icon);
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

        let legend_items = if self.items.is_empty() {
            chart_context(cx)
                .map(|ctx| {
                    ctx.config
                        .iter()
                        .map(|(key, item)| {
                            let label = item.label.clone().unwrap_or_else(|| key.clone());
                            let mut legend_item = ChartLegendItem::new(label);
                            if let Some(color) = item.color.clone() {
                                legend_item = legend_item.color(color);
                            }
                            if let Some(icon) = item.icon.clone() {
                                legend_item = legend_item.icon(icon);
                            }
                            legend_item
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            self.items
        };

        let items = legend_items
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
                if !self.hide_icon {
                    if let Some(icon_id) = item.icon {
                        row.push(crate::raw::icon::icon(cx, icon_id));
                    } else {
                        row.push(indicator);
                    }
                } else {
                    row.push(indicator);
                }
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
