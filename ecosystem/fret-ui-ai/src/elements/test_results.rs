//! AI Elements-aligned `TestResults` surfaces.

use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextOverflow, TextStyle,
    TextWrap, Transform2D,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps,
    SemanticsDecoration, SemanticsProps, SizeStyle, TextProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius,
    Space,
};
use fret_ui_shadcn::{Collapsible, CollapsibleContent, CollapsibleTrigger};

pub type OnTestActivate = Arc<
    dyn Fn(&mut dyn fret_ui::action::UiActionHost, fret_ui::action::ActionCx, Arc<str>) + 'static,
>;

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    }
}

fn format_duration(ms: u64) -> Arc<str> {
    if ms < 1000 {
        return Arc::<str>::from(format!("{ms}ms"));
    }
    Arc::<str>::from(format!("{:.2}s", (ms as f32) / 1000.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestStatusKind {
    Passed,
    Failed,
    Skipped,
    Running,
}

impl TestStatusKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::Running => "running",
        }
    }

    pub fn icon_id(self) -> fret_icons::IconId {
        match self {
            Self::Passed => ids::ui::STATUS_SUCCEEDED,
            Self::Failed => ids::ui::STATUS_FAILED,
            Self::Skipped => ids::ui::STATUS_PENDING,
            Self::Running => ids::ui::STATUS_RUNNING,
        }
    }

    pub fn color(self, theme: &Theme) -> Color {
        match self {
            Self::Failed => theme
                .color_by_key("destructive")
                .unwrap_or_else(|| theme.color_required("foreground")),
            Self::Passed => Color {
                r: 0.086,
                g: 0.639,
                b: 0.290,
                a: 1.0,
            },
            Self::Skipped => Color {
                r: 0.792,
                g: 0.541,
                b: 0.016,
                a: 1.0,
            },
            Self::Running => Color {
                r: 0.145,
                g: 0.388,
                b: 0.922,
                a: 1.0,
            },
        }
    }
}

impl std::fmt::Display for TestStatusKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct TestResultsSummaryData {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub total: u32,
    pub duration_ms: Option<u64>,
}

impl TestResultsSummaryData {
    pub fn new(passed: u32, failed: u32, skipped: u32, total: u32) -> Self {
        Self {
            passed,
            failed,
            skipped,
            total,
            duration_ms: None,
        }
    }

    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }
}

fn status_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    status: TestStatusKind,
    label: Arc<str>,
) -> AnyElement {
    let fg = status.color(theme);
    let bg = alpha(fg, 0.18);

    let text_px = theme
        .metric_by_key("component.badge.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));

    let mut props = ContainerProps::default();
    props.padding = Edges::symmetric(
        MetricRef::space(Space::N2).resolve(theme),
        MetricRef::space(Space::N0p5).resolve(theme),
    );
    props.background = Some(bg);
    props.corner_radii = Corners::all(MetricRef::radius(Radius::Full).resolve(theme));

    cx.container(props, move |cx| {
        let icon = decl_icon::icon_with(
            cx,
            status.icon_id(),
            Some(Px(12.0)),
            Some(ColorRef::Color(fg)),
        );
        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: label.clone(),
            style: Some(TextStyle {
                line_height: Some(line_height),
                ..monospace_style(theme, text_px, FontWeight::SEMIBOLD)
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        vec![stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_center()
                .justify_center()
                .gap(Space::N1),
            move |_cx| vec![icon, text],
        )]
    })
}

/// Root surface aligned with AI Elements `TestResults`.
#[derive(Clone)]
pub struct TestResults {
    summary: Option<TestResultsSummaryData>,
    children: Option<Vec<AnyElement>>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TestResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestResults")
            .field("has_summary", &self.summary.is_some())
            .field(
                "children_len",
                &self.children.as_ref().map(|c| c.len()).unwrap_or(0),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            summary: None,
            children: None,
            test_id_root: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn summary(mut self, summary: TestResultsSummaryData) -> Self {
        self.summary = Some(summary);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id_root(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let chrome = base_chrome.merge(self.chrome);
        let layout = LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .merge(self.layout);

        let children = if let Some(children) = self.children {
            children
        } else if let Some(summary) = self.summary.clone() {
            vec![
                TestResultsHeader::new([
                    TestResultsSummary::new(summary.clone()).into_element(cx),
                    TestResultsDuration::new(summary).into_element(cx),
                ])
                .into_element(cx),
            ]
        } else {
            Vec::new()
        };

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |_cx| children,
        );

        let Some(test_id) = self.test_id_root else {
            return root;
        };
        root.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Header wrapper aligned with AI Elements `TestResultsHeader`.
#[derive(Debug, Clone)]
pub struct TestResultsHeader {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .justify(Justify::Between)
                .items_center(),
            move |_cx| self.children,
        );

        let mut props =
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default());
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(border);

        let header = cx.container(props, move |_cx| vec![row]);

        let Some(test_id) = self.test_id else {
            return header;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![header],
        )
    }
}

/// Summary badges aligned with AI Elements `TestResultsSummary`.
#[derive(Debug, Clone)]
pub struct TestResultsSummary {
    summary: TestResultsSummaryData,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsSummary {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary,
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut badges: Vec<AnyElement> = Vec::new();
        badges.push(status_badge(
            cx,
            &theme,
            TestStatusKind::Passed,
            Arc::<str>::from(format!("{} passed", self.summary.passed)),
        ));

        if self.summary.failed > 0 {
            badges.push(status_badge(
                cx,
                &theme,
                TestStatusKind::Failed,
                Arc::<str>::from(format!("{} failed", self.summary.failed)),
            ));
        }

        if self.summary.skipped > 0 {
            badges.push(status_badge(
                cx,
                &theme,
                TestStatusKind::Skipped,
                Arc::<str>::from(format!("{} skipped", self.summary.skipped)),
            ));
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N3)
                .items_center(),
            move |_cx| badges,
        );

        cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![row],
        )
    }
}

/// Duration label aligned with AI Elements `TestResultsDuration`.
#[derive(Debug, Clone)]
pub struct TestResultsDuration {
    summary: TestResultsSummaryData,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsDuration {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary,
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let Some(ms) = self.summary.duration_ms else {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        };

        let label = format_duration(ms);
        let fg = theme.color_required("muted-foreground");

        let text = cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, self.layout),
            text: label,
            style: None,
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![text],
        )
    }
}

/// Progress bar aligned with AI Elements `TestResultsProgress`.
#[derive(Debug, Clone)]
pub struct TestResultsProgress {
    summary: TestResultsSummaryData,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsProgress {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        if self.summary.total == 0 {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        }

        let passed = self.summary.passed as f32;
        let failed = self.summary.failed as f32;
        let total = self.summary.total as f32;
        let passed_percent = ((passed / total) * 100.0).clamp(0.0, 100.0);

        let bar_bg = theme.color_required("muted");
        let passed_color = TestStatusKind::Passed.color(&theme);
        let failed_color = TestStatusKind::Failed.color(&theme);

        let bar_h = Px(8.0);
        let radius = MetricRef::radius(Radius::Full).resolve(&theme);

        let bar = {
            let theme_for_bar = theme.clone();
            let mut bar_props = ContainerProps::default();
            bar_props.layout = {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Px(bar_h);
                layout.overflow = fret_ui::element::Overflow::Clip;
                layout
            };
            bar_props.background = Some(bar_bg);
            bar_props.corner_radii = Corners::all(radius);

            cx.container(bar_props, move |cx| {
                let passed_seg = {
                    let mut props = ContainerProps::default();
                    props.layout = decl_style::layout_style(
                        &theme_for_bar,
                        LayoutRefinement::default()
                            .h_full()
                            .basis_0()
                            .flex_grow(passed.max(0.0)),
                    );
                    props.background = Some(passed_color);
                    cx.container(props, |_cx| Vec::new())
                };

                let failed_seg = {
                    let mut props = ContainerProps::default();
                    props.layout = decl_style::layout_style(
                        &theme_for_bar,
                        LayoutRefinement::default()
                            .h_full()
                            .basis_0()
                            .flex_grow(failed.max(0.0)),
                    );
                    props.background = Some(failed_color);
                    cx.container(props, |_cx| Vec::new())
                };

                vec![stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N0)
                        .items(Items::Stretch),
                    move |_cx| vec![passed_seg, failed_seg],
                )]
            })
        };

        let text_left: Arc<str> = Arc::from(format!(
            "{}/{} tests passed",
            self.summary.passed, self.summary.total
        ));
        let text_right: Arc<str> = Arc::from(format!("{:.0}%", passed_percent));
        let fg = theme.color_required("muted-foreground");

        let labels = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .justify(Justify::Between)
                .items_center(),
            move |cx| {
                vec![
                    cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: text_left.clone(),
                        style: None,
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }),
                    cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: text_right.clone(),
                        style: None,
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    }),
                ]
            },
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items(Items::Stretch),
            move |_cx| vec![bar, labels],
        );

        let wrapper = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![content],
        );

        let Some(test_id) = self.test_id else {
            return wrapper;
        };
        wrapper.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Content wrapper aligned with AI Elements `TestResultsContent`.
#[derive(Debug, Clone)]
pub struct TestResultsContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

fn status_icon<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    status: TestStatusKind,
    size: Px,
) -> AnyElement {
    let fg = status.color(theme);
    decl_icon::icon_with(cx, status.icon_id(), Some(size), Some(ColorRef::Color(fg)))
}

/// Test suite disclosure root aligned with AI Elements `TestSuite`.
#[derive(Debug, Clone)]
pub struct TestSuite {
    default_open: bool,
    header: TestSuiteName,
    content: TestSuiteContent,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestSuite {
    pub fn new(header: TestSuiteName, content: TestSuiteContent) -> Self {
        Self {
            default_open: false,
            header,
            content,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let header = self.header;
        let content = self.content;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout)
            .refine_style(base_chrome.merge(self.chrome))
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| header.clone().into_trigger(cx, open_model, is_open),
                move |cx| content.clone().into_element(cx),
            )
    }
}

/// Collapsible trigger row aligned with AI Elements `TestSuiteName`.
#[derive(Clone)]
pub struct TestSuiteName {
    name: Arc<str>,
    status: TestStatusKind,
    passed: u32,
    failed: u32,
    skipped: u32,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TestSuiteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSuiteName")
            .field("name", &self.name.as_ref())
            .field("status", &self.status)
            .field("passed", &self.passed)
            .field("failed", &self.failed)
            .field("skipped", &self.skipped)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl TestSuiteName {
    pub fn new(name: impl Into<Arc<str>>, status: TestStatusKind) -> Self {
        Self {
            name: name.into(),
            status,
            passed: 0,
            failed: 0,
            skipped: 0,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
        }
    }

    pub fn stats(mut self, passed: u32, failed: u32, skipped: u32) -> Self {
        self.passed = passed;
        self.failed = failed;
        self.skipped = skipped;
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    fn into_trigger<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open_model: Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let chevron_size = Px(16.0);
        let center = Point::new(Px(8.0), Px(8.0));
        let chevron_rotation = if is_open { 90.0 } else { 0.0 };
        let chevron_transform = Transform2D::rotation_about_degrees(chevron_rotation, center);
        let chevron_fg = theme.color_required("muted-foreground");
        let chevron = cx.visual_transform_props(
            VisualTransformProps {
                layout: decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .w_px(MetricRef::Px(chevron_size))
                        .h_px(MetricRef::Px(chevron_size))
                        .flex_shrink_0(),
                ),
                transform: chevron_transform,
            },
            move |cx| {
                vec![decl_icon::icon_with(
                    cx,
                    ids::ui::CHEVRON_RIGHT,
                    Some(chevron_size),
                    Some(ColorRef::Color(chevron_fg)),
                )]
            },
        );

        let status_icon = status_icon(cx, &theme, self.status, Px(16.0));

        let name_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.name.clone(),
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let stats = {
            let mut parts: Vec<AnyElement> = Vec::new();
            if self.passed > 0 {
                parts.push(cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{} passed", self.passed)),
                    style: None,
                    color: Some(TestStatusKind::Passed.color(&theme)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }));
            }
            if self.failed > 0 {
                parts.push(cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{} failed", self.failed)),
                    style: None,
                    color: Some(TestStatusKind::Failed.color(&theme)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }));
            }
            if self.skipped > 0 {
                parts.push(cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{} skipped", self.skipped)),
                    style: None,
                    color: Some(TestStatusKind::Skipped.color(&theme)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }));
            }

            if parts.is_empty() {
                None
            } else {
                Some(stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().flex_shrink_0())
                        .gap(Space::N2)
                        .items_center(),
                    move |_cx| parts,
                ))
            }
        };

        let mut left: Vec<AnyElement> = vec![chevron, status_icon, name_text];
        if let Some(stats) = stats {
            left.push(stats);
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N2)
                .items_center(),
            move |_cx| left,
        );

        let row = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![row],
        );

        let trigger = CollapsibleTrigger::new(open_model, vec![row])
            .a11y_label("Toggle test suite")
            .into_element(cx, is_open);

        let Some(test_id) = self.test_id else {
            return trigger;
        };
        trigger.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .test_id(test_id),
        )
    }
}

/// Collapsible content wrapper aligned with AI Elements `TestSuiteContent`.
#[derive(Debug, Clone)]
pub struct TestSuiteContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

/// Test row aligned with AI Elements `Test`.
#[derive(Clone)]
pub struct Test {
    name: Arc<str>,
    status: TestStatusKind,
    duration_ms: Option<u32>,
    children: Option<Vec<AnyElement>>,
    on_activate: Option<OnTestActivate>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Test {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Test")
            .field("name", &self.name.as_ref())
            .field("status", &self.status)
            .field("duration_ms", &self.duration_ms)
            .field(
                "children_len",
                &self.children.as_ref().map(|c| c.len()).unwrap_or(0),
            )
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Test {
    pub fn new(name: impl Into<Arc<str>>, status: TestStatusKind) -> Self {
        Self {
            name: name.into(),
            status,
            duration_ms: None,
            children: None,
            on_activate: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N2),
        }
    }

    pub fn duration_ms(mut self, duration_ms: u32) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    /// Optional click/activate seam for app-owned effects (e.g. open test output).
    pub fn on_activate(mut self, on_activate: OnTestActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let status = self.status;
        let name = self.name;
        let duration = self.duration_ms;

        let layout = self.layout;
        let chrome = self.chrome;
        let test_id = self.test_id;

        let theme_for_content = theme.clone();
        let name_for_content = name.clone();
        let content_factory = move |cx: &mut ElementContext<'_, H>| {
            if let Some(children) = self.children.clone() {
                return stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2)
                        .items_center(),
                    move |_cx| children,
                );
            }

            let status_el = status_icon(cx, &theme_for_content, status, Px(16.0));
            let name_el = cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        min_width: Some(Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: name_for_content.clone(),
                style: None,
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
            });

            let duration_el = duration.map(|ms| {
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("{ms}ms")),
                    style: None,
                    color: Some(theme_for_content.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
            });

            let mut children = vec![status_el, name_el];
            if let Some(duration_el) = duration_el {
                children.push(duration_el);
            }

            stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .items_center(),
                move |_cx| children,
            )
        };

        let Some(on_activate) = self.on_activate else {
            let content = content_factory(cx);
            let el = cx.container(
                decl_style::container_props(&theme, chrome, layout),
                move |_cx| vec![content],
            );
            let Some(test_id) = test_id else {
                return el;
            };
            return el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            );
        };

        let hover_bg = theme
            .color_by_key("muted")
            .map(|c| alpha(c, 0.5))
            .unwrap_or_else(|| alpha(theme.color_required("accent"), 0.2));
        let pressed_bg = theme
            .color_by_key("accent")
            .map(|c| alpha(c, 0.35))
            .unwrap_or_else(|| alpha(theme.color_required("muted"), 0.8));

        let label_name = name.clone();
        let mut pressable = PressableProps::default();
        pressable.enabled = true;
        pressable.focusable = true;
        pressable.a11y = PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(Arc::<str>::from("Open test output")),
            test_id: test_id.clone(),
            ..Default::default()
        };

        cx.pressable(pressable, move |cx, st| {
            cx.pressable_on_activate({
                let on_activate = on_activate.clone();
                let label_name = label_name.clone();
                Arc::new(move |host, action_cx, _reason| {
                    on_activate(host, action_cx, label_name.clone());
                })
            });

            let bg = if st.pressed {
                Some(pressed_bg)
            } else if st.hovered {
                Some(hover_bg)
            } else {
                None
            };

            let content = content_factory(cx);
            let mut props = decl_style::container_props(&theme, chrome, layout);
            if let Some(bg) = bg {
                props.background = Some(bg);
            }
            vec![cx.container(props, move |_cx| vec![content])]
        })
    }
}

/// Error wrapper aligned with AI Elements `TestError`.
#[derive(Debug, Clone)]
pub struct TestError {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestError {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().p(Space::N3).rounded(Radius::Md),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let destructive = theme
            .color_by_key("destructive")
            .unwrap_or_else(|| theme.color_required("foreground"));
        let bg = alpha(destructive, 0.12);

        let chrome = self.chrome.bg(ColorRef::Color(bg));
        let children = self.children;
        cx.container(
            decl_style::container_props(&theme, chrome, self.layout),
            move |_cx| children,
        )
    }
}

/// Error message aligned with AI Elements `TestErrorMessage`.
#[derive(Debug, Clone)]
pub struct TestErrorMessage {
    text: Arc<str>,
    layout: LayoutRefinement,
}

impl TestErrorMessage {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = TestStatusKind::Failed.color(&theme);

        cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, self.layout),
            text: self.text,
            style: None,
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

/// Error stack aligned with AI Elements `TestErrorStack`.
#[derive(Debug, Clone)]
pub struct TestErrorStack {
    text: Arc<str>,
    max_height: Option<Px>,
    layout: LayoutRefinement,
}

impl TestErrorStack {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            max_height: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn max_height(mut self, max_height: Px) -> Self {
        self.max_height = Some(Px(max_height.0.max(0.0)));
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = TestStatusKind::Failed.color(&theme);
        let text_px = theme
            .metric_by_key("fret.ai.test_results.error_stack.text_px")
            .or_else(|| theme.metric_by_key("component.code_block.text_px"))
            .unwrap_or(Px(11.0));
        let style = monospace_style(&theme, text_px, FontWeight::NORMAL);

        let text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(style),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let mut scroll = fret_ui_shadcn::ScrollArea::new([text])
            .axis(fret_ui::element::ScrollAxis::Both)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0());
        if let Some(max_height) = self.max_height {
            scroll =
                scroll.refine_layout(LayoutRefinement::default().max_h(MetricRef::Px(max_height)));
        }
        let scroll = scroll.into_element(cx);

        cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![scroll],
        )
    }
}

impl TestSuiteContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let border = theme.color_required("border");

        let children = self.children;
        let divided: Vec<AnyElement> = children
            .into_iter()
            .enumerate()
            .map(|(i, child)| {
                if i == 0 {
                    return child;
                }

                let mut props = ContainerProps::default();
                props.layout =
                    decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
                props.border = Edges {
                    top: Px(1.0),
                    right: Px(0.0),
                    bottom: Px(0.0),
                    left: Px(0.0),
                };
                props.border_color = Some(border);
                cx.container(props, move |_cx| vec![child])
            })
            .collect();

        let wrapper = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |cx| {
                let content = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(self.layout)
                        .gap(Space::N0)
                        .items(Items::Stretch),
                    move |_cx| divided,
                );
                vec![content]
            },
        );

        let wrapper = if let Some(test_id) = self.test_id {
            wrapper.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            wrapper
        };

        let mut chrome = ChromeRefinement::default();
        chrome = chrome.border_1().border_color(ColorRef::Color(border));
        let mut props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        props.border = Edges {
            top: Px(1.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        };
        props.border_color = Some(border);

        let inner = cx.container(props, move |_cx| vec![wrapper]);

        CollapsibleContent::new([inner]).into_element(cx)
    }
}

impl TestResultsContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().p(Space::N4),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let children = self.children;
        let layout = self.layout;
        let content = cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |cx| {
                vec![stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(layout)
                        .gap(Space::N2)
                        .items(Items::Stretch),
                    move |_cx| children,
                )]
            },
        );

        let Some(test_id) = self.test_id else {
            return content;
        };
        content.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}
