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
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius,
    Space,
};
use fret_ui_shadcn::facade::{Collapsible, CollapsibleContent, CollapsibleTrigger, ScrollArea};

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
    typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    })
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
        fn token(theme: &Theme, key: &'static str, fallback: Color) -> Color {
            theme.color_by_key(key).unwrap_or(fallback)
        }

        match self {
            Self::Failed => theme
                .color_by_key("component.test_results.status.failed")
                .unwrap_or_else(|| {
                    theme
                        .color_by_key("destructive")
                        .unwrap_or_else(|| theme.color_token("foreground"))
                }),
            Self::Passed => token(
                theme,
                "component.test_results.status.passed",
                // Tailwind: green-600 (#16a34a).
                fret_ui_kit::colors::linear_from_hex_rgb(0x16_a3_4a),
            ),
            Self::Skipped => token(
                theme,
                "component.test_results.status.skipped",
                // Tailwind: yellow-600 (#ca8a04).
                fret_ui_kit::colors::linear_from_hex_rgb(0xca_8a_04),
            ),
            Self::Running => token(
                theme,
                "component.test_results.status.running",
                // Tailwind: blue-600 (#2563eb).
                fret_ui_kit::colors::linear_from_hex_rgb(0x25_63_eb),
            ),
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

fn use_test_results_summary<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<TestResultsSummaryData> {
    cx.provided::<TestResultsSummaryData>().cloned()
}

#[derive(Debug, Clone)]
struct TestSuiteContextData {
    name: Arc<str>,
    status: TestStatusKind,
}

fn use_test_suite_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TestSuiteContextData> {
    cx.provided::<TestSuiteContextData>().cloned()
}

#[derive(Debug, Clone)]
struct TestContextData {
    name: Arc<str>,
    status: TestStatusKind,
    duration_ms: Option<u32>,
}

fn use_test_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<TestContextData> {
    cx.provided::<TestContextData>().cloned()
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
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.badge.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    let mut props = ContainerProps::default();
    props.padding = Edges::symmetric(
        MetricRef::space(Space::N2).resolve(theme),
        MetricRef::space(Space::N0p5).resolve(theme),
    )
    .into();
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
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        vec![
            ui::h_row(move |_cx| vec![icon, text])
                .items(Items::Center)
                .justify(Justify::Center)
                .gap(Space::N1)
                .into_element(cx),
        ]
    })
}

/// Root surface aligned with AI Elements `TestResults`.
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

        let summary = self.summary.clone();
        let explicit_children = self.children;

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                let render_children = |cx: &mut ElementContext<'_, H>| {
                    if let Some(children) = explicit_children {
                        return children;
                    }

                    if summary.is_none() {
                        return Vec::new();
                    }

                    vec![
                        TestResultsHeader::new([
                            TestResultsSummary::from_context().into_element(cx),
                            TestResultsDuration::from_context().into_element(cx),
                        ])
                        .into_element(cx),
                    ]
                };

                if let Some(summary) = summary.clone() {
                    cx.provide(summary, render_children)
                } else {
                    render_children(cx)
                }
            },
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
#[derive(Debug)]
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
        let border = theme.color_token("border");

        let row = ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .gap(Space::N2)
            .justify(Justify::Between)
            .items(Items::Center)
            .into_element(cx);

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
    summary: Option<TestResultsSummaryData>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsSummary {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary: Some(summary),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            summary: None,
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
        let Some(summary) = self.summary.or_else(|| use_test_results_summary(cx)) else {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        };

        let mut badges: Vec<AnyElement> = Vec::new();
        badges.push(status_badge(
            cx,
            &theme,
            TestStatusKind::Passed,
            Arc::<str>::from(format!("{} passed", summary.passed)),
        ));

        if summary.failed > 0 {
            badges.push(status_badge(
                cx,
                &theme,
                TestStatusKind::Failed,
                Arc::<str>::from(format!("{} failed", summary.failed)),
            ));
        }

        if summary.skipped > 0 {
            badges.push(status_badge(
                cx,
                &theme,
                TestStatusKind::Skipped,
                Arc::<str>::from(format!("{} skipped", summary.skipped)),
            ));
        }

        let row = ui::h_row(move |_cx| badges)
            .layout(self.layout)
            .gap(Space::N3)
            .items(Items::Center)
            .into_element(cx);

        cx.container(
            decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
            move |_cx| vec![row],
        )
    }
}

/// Duration label aligned with AI Elements `TestResultsDuration`.
#[derive(Debug, Clone)]
pub struct TestResultsDuration {
    summary: Option<TestResultsSummaryData>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsDuration {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary: Some(summary),
            layout: LayoutRefinement::default().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            summary: None,
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
        let Some(summary) = self.summary.or_else(|| use_test_results_summary(cx)) else {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        };
        let Some(ms) = summary.duration_ms else {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        };

        let label = format_duration(ms);
        let fg = theme.color_token("muted-foreground");

        let text = cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, self.layout),
            text: label,
            style: None,
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
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
    summary: Option<TestResultsSummaryData>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestResultsProgress {
    pub fn new(summary: TestResultsSummaryData) -> Self {
        Self {
            summary: Some(summary),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            summary: None,
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
        let Some(summary) = self.summary.or_else(|| use_test_results_summary(cx)) else {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        };
        if summary.total == 0 {
            return cx.container(
                decl_style::container_props(&theme, self.chrome, LayoutRefinement::default()),
                |_cx| Vec::new(),
            );
        }

        let passed = summary.passed as f32;
        let failed = summary.failed as f32;
        let total = summary.total as f32;
        let passed_percent = ((passed / total) * 100.0).clamp(0.0, 100.0);

        let bar_bg = theme.color_token("muted");
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

                vec![
                    ui::h_row(move |_cx| vec![passed_seg, failed_seg])
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N0)
                        .items(Items::Stretch)
                        .into_element(cx),
                ]
            })
        };

        let text_left: Arc<str> =
            Arc::from(format!("{}/{} tests passed", summary.passed, summary.total));
        let text_right: Arc<str> = Arc::from(format!("{:.0}%", passed_percent));
        let fg = theme.color_token("muted-foreground");

        let labels = ui::h_row(move |cx| {
            vec![
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: text_left.clone(),
                    style: None,
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: text_right.clone(),
                    style: None,
                    color: Some(fg),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                }),
            ]
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N2)
        .justify(Justify::Between)
        .items(Items::Center)
        .into_element(cx);

        let content = ui::v_stack(move |_cx| vec![bar, labels])
            .layout(self.layout)
            .gap(Space::N2)
            .items(Items::Stretch)
            .into_element(cx);

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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct TestSuite {
    default_open: bool,
    name: Arc<str>,
    status: TestStatusKind,
    trigger: Option<TestSuiteName>,
    content: Option<TestSuiteContent>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl TestSuite {
    pub fn new(trigger: TestSuiteName, content: TestSuiteContent) -> Self {
        let name = trigger.name.clone().unwrap_or_default();
        let status = trigger.status.unwrap_or(TestStatusKind::Passed);
        Self {
            default_open: false,
            name,
            status,
            trigger: Some(trigger),
            content: Some(content),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn named(name: impl Into<Arc<str>>, status: TestStatusKind) -> Self {
        Self {
            default_open: false,
            name: name.into(),
            status,
            trigger: None,
            content: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn trigger(mut self, trigger: TestSuiteName) -> Self {
        self.trigger = Some(trigger);
        self
    }

    pub fn content(mut self, content: TestSuiteContent) -> Self {
        self.content = Some(content);
        self
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

        let suite_context = TestSuiteContextData {
            name: self.name,
            status: self.status,
        };
        let trigger = self.trigger.unwrap_or_else(TestSuiteName::from_context);
        let content = self
            .content
            .unwrap_or_else(|| TestSuiteContent::new(Vec::<AnyElement>::new()));
        let suite_context_for_content = suite_context.clone();

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout)
            .refine_style(base_chrome.merge(self.chrome))
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    cx.provide(suite_context.clone(), |cx| {
                        trigger.into_trigger(cx, open_model, is_open)
                    })
                },
                move |cx| {
                    cx.provide(suite_context_for_content.clone(), |cx| {
                        content.into_element(cx)
                    })
                },
            )
    }
}

/// Collapsible trigger row aligned with AI Elements `TestSuiteName`.
pub struct TestSuiteName {
    name: Option<Arc<str>>,
    status: Option<TestStatusKind>,
    passed: u32,
    failed: u32,
    skipped: u32,
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for TestSuiteName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestSuiteName")
            .field("name", &self.name.as_deref())
            .field("status", &self.status)
            .field("passed", &self.passed)
            .field("failed", &self.failed)
            .field("skipped", &self.skipped)
            .field(
                "children_len",
                &self.children.as_ref().map(|c| c.len()).unwrap_or(0),
            )
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl TestSuiteName {
    pub fn new(name: impl Into<Arc<str>>, status: TestStatusKind) -> Self {
        Self {
            name: Some(name.into()),
            status: Some(status),
            passed: 0,
            failed: 0,
            skipped: 0,
            children: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
        }
    }

    pub fn from_context() -> Self {
        Self {
            name: None,
            status: None,
            passed: 0,
            failed: 0,
            skipped: 0,
            children: None,
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

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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
        let suite_context = use_test_suite_context(cx);
        let name = self
            .name
            .clone()
            .or_else(|| suite_context.as_ref().map(|ctx| ctx.name.clone()))
            .unwrap_or_default();
        let status = self
            .status
            .or_else(|| suite_context.as_ref().map(|ctx| ctx.status))
            .unwrap_or(TestStatusKind::Passed);

        let chevron_size = Px(16.0);
        let center = Point::new(Px(8.0), Px(8.0));
        let chevron_rotation = if is_open { 90.0 } else { 0.0 };
        let chevron_transform = Transform2D::rotation_about_degrees(chevron_rotation, center);
        let chevron_fg = theme.color_token("muted-foreground");
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

        let status_icon = status_icon(cx, &theme, status, Px(16.0));

        let name_text = if let Some(children) = self.children {
            ui::h_row(move |_cx| children)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items(Items::Center)
                .into_element(cx)
        } else {
            cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        min_width: Some(Length::Px(Px(0.0))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: name,
                style: None,
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Ellipsis,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            })
        };

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
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
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
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
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
                    align: fret_core::TextAlign::Start,
                    ink_overflow: Default::default(),
                }));
            }

            if parts.is_empty() {
                None
            } else {
                Some(
                    ui::h_row(move |_cx| parts)
                        .layout(LayoutRefinement::default().ml_auto().flex_shrink_0())
                        .gap(Space::N2)
                        .items(Items::Center)
                        .into_element(cx),
                )
            }
        };

        let mut left: Vec<AnyElement> = vec![chevron, status_icon, name_text];
        if let Some(stats) = stats {
            left.push(stats);
        }

        let row = ui::h_row(move |_cx| left)
            .layout(self.layout)
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx);

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

/// Optional suite stats part aligned with AI Elements `TestSuiteStats`.
#[derive(Debug, Clone)]
pub struct TestSuiteStats {
    passed: u32,
    failed: u32,
    skipped: u32,
    layout: LayoutRefinement,
}

impl TestSuiteStats {
    pub fn new(passed: u32, failed: u32, skipped: u32) -> Self {
        Self {
            passed,
            failed,
            skipped,
            layout: LayoutRefinement::default().ml_auto().flex_shrink_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let mut parts: Vec<AnyElement> = Vec::new();
        if self.passed > 0 {
            parts.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from(format!("{} passed", self.passed)),
                style: None,
                color: Some(TestStatusKind::Passed.color(&theme)),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
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
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
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
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            }));
        }

        ui::h_row(move |_cx| parts)
            .layout(self.layout)
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx)
    }
}

/// Collapsible content wrapper aligned with AI Elements `TestSuiteContent`.
#[derive(Debug)]
pub struct TestSuiteContent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

/// Test row aligned with AI Elements `Test`.
pub struct Test {
    name: Arc<str>,
    status: TestStatusKind,
    duration_ms: Option<u32>,
    children: Option<Vec<AnyElement>>,
    details: Option<Vec<AnyElement>>,
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
            .field(
                "details_len",
                &self.details.as_ref().map(|c| c.len()).unwrap_or(0),
            )
            .field("has_on_activate", &self.on_activate.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

/// Status icon part aligned with AI Elements `TestStatus`.
#[derive(Debug, Clone)]
pub struct TestStatus {
    status: Option<TestStatusKind>,
    layout: LayoutRefinement,
}

impl TestStatus {
    pub fn new(status: TestStatusKind) -> Self {
        Self {
            status: Some(status),
            layout: LayoutRefinement::default().flex_shrink_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            status: None,
            layout: LayoutRefinement::default().flex_shrink_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let status = self
            .status
            .or_else(|| use_test_context(cx).map(|ctx| ctx.status))
            .unwrap_or(TestStatusKind::Passed);

        cx.container(
            ContainerProps {
                layout: decl_style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |cx| vec![status_icon(cx, &theme, status, Px(16.0))],
        )
    }
}

/// Name label part aligned with AI Elements `TestName`.
#[derive(Debug, Clone)]
pub struct TestName {
    name: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl TestName {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: Some(name.into()),
            layout: LayoutRefinement::default().flex_1().min_w_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            name: None,
            layout: LayoutRefinement::default().flex_1().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let name = self
            .name
            .or_else(|| use_test_context(cx).map(|ctx| ctx.name))
            .unwrap_or_default();

        cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, self.layout),
            text: name,
            style: None,
            color: None,
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

/// Duration label part aligned with AI Elements `TestDuration`.
#[derive(Debug, Clone)]
pub struct TestDuration {
    duration_ms: Option<u32>,
    layout: LayoutRefinement,
}

impl TestDuration {
    pub fn new(duration_ms: u32) -> Self {
        Self {
            duration_ms: Some(duration_ms),
            layout: LayoutRefinement::default().ml_auto().flex_shrink_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            duration_ms: None,
            layout: LayoutRefinement::default().ml_auto().flex_shrink_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let duration_ms = self
            .duration_ms
            .or_else(|| use_test_context(cx).and_then(|ctx| ctx.duration_ms));
        let Some(duration_ms) = duration_ms else {
            return cx.container(
                ContainerProps {
                    layout: decl_style::layout_style(&theme, self.layout),
                    ..Default::default()
                },
                |_cx| Vec::new(),
            );
        };

        cx.text_props(TextProps {
            layout: decl_style::layout_style(&theme, self.layout),
            text: Arc::<str>::from(format!("{duration_ms}ms")),
            style: None,
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        })
    }
}

impl Test {
    pub fn new(name: impl Into<Arc<str>>, status: TestStatusKind) -> Self {
        Self {
            name: name.into(),
            status,
            duration_ms: None,
            children: None,
            details: None,
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

    pub fn details(mut self, details: impl IntoIterator<Item = AnyElement>) -> Self {
        self.details = Some(details.into_iter().collect());
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
        let test_context = TestContextData {
            name: name.clone(),
            status,
            duration_ms: duration,
        };

        let layout = self.layout;
        let chrome = self.chrome;
        let test_id = self.test_id;

        let children = self.children;
        let details = self.details;
        let content_factory = move |cx: &mut ElementContext<'_, H>| {
            cx.provide(test_context.clone(), |cx| {
                let content = if let Some(children) = children {
                    ui::h_row(move |_cx| children)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2)
                        .items(Items::Center)
                        .into_element(cx)
                } else {
                    ui::h_row(move |cx| {
                        vec![
                            TestStatus::from_context().into_element(cx),
                            TestName::from_context().into_element(cx),
                            TestDuration::from_context().into_element(cx),
                        ]
                    })
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .items(Items::Center)
                    .into_element(cx)
                };

                let Some(details) = details else {
                    return content;
                };

                let mut stacked = vec![content];
                stacked.extend(details);
                ui::v_stack(move |_cx| stacked)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .items(Items::Stretch)
                    .into_element(cx)
            })
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
            .unwrap_or_else(|| alpha(theme.color_token("accent"), 0.2));
        let pressed_bg = theme
            .color_by_key("accent")
            .map(|c| alpha(c, 0.35))
            .unwrap_or_else(|| alpha(theme.color_token("muted"), 0.8));

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
#[derive(Debug)]
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
            .unwrap_or_else(|| theme.color_token("foreground"));
        let bg = alpha(destructive, 0.12);

        let chrome = self.chrome.bg(ColorRef::Color(bg));
        let children = self.children;
        let content = ui::v_stack(move |_cx| children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Stretch)
            .into_element(cx);

        cx.container(
            decl_style::container_props(&theme, chrome, self.layout),
            move |_cx| vec![content],
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
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
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
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let mut scroll = ScrollArea::new([text])
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
        let border = theme.color_token("border");

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
                let content = ui::v_stack(move |_cx| divided)
                    .layout(self.layout)
                    .gap(Space::N0)
                    .items(Items::Stretch)
                    .into_element(cx);
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
                vec![
                    ui::v_stack(move |_cx| children)
                        .layout(layout)
                        .gap(Space::N2)
                        .items(Items::Stretch)
                        .into_element(cx),
                ]
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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(640.0), Px(360.0)),
        )
    }

    fn find_text_by_content<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Text(props) = &element.kind
            && props.text.as_ref() == needle
        {
            return Some(element);
        }

        element
            .children
            .iter()
            .find_map(|child| find_text_by_content(child, needle))
    }

    #[test]
    fn summary_parts_can_defer_to_root_context() {
        let summary = TestResultsSummary::from_context();
        let duration = TestResultsDuration::from_context();
        let progress = TestResultsProgress::from_context();

        assert!(summary.summary.is_none());
        assert!(duration.summary.is_none());
        assert!(progress.summary.is_none());
    }

    #[test]
    fn suite_named_constructor_keeps_part_context() {
        let suite = TestSuite::named("Auth", TestStatusKind::Passed)
            .trigger(TestSuiteName::from_context())
            .content(TestSuiteContent::new(Vec::<AnyElement>::new()));

        assert_eq!(suite.name.as_ref(), "Auth");
        assert_eq!(suite.status, TestStatusKind::Passed);
        assert!(suite.trigger.is_some());
        assert!(suite.content.is_some());
    }

    #[test]
    fn test_row_parts_can_defer_to_test_context() {
        let status = TestStatus::from_context();
        let name = TestName::from_context();
        let duration = TestDuration::from_context();

        assert!(status.status.is_none());
        assert!(name.name.is_none());
        assert!(duration.duration_ms.is_none());
    }

    #[test]
    fn suite_name_from_context_starts_without_explicit_values() {
        let trigger = TestSuiteName::from_context();

        assert!(trigger.name.is_none());
        assert!(trigger.status.is_none());
        assert!(trigger.children.is_none());
    }

    #[test]
    fn test_results_root_provides_summary_to_context_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let summary = TestResultsSummaryData::new(3, 1, 2, 6).duration_ms(1234);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                TestResults::new().summary(summary.clone()).into_element(cx)
            });

        assert!(find_text_by_content(&element, "3 passed").is_some());
        assert!(find_text_by_content(&element, "1 failed").is_some());
        assert!(find_text_by_content(&element, "2 skipped").is_some());
        assert!(find_text_by_content(&element, "1.23s").is_some());
    }

    #[test]
    fn test_suite_root_provides_context_to_trigger_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                TestSuite::named("Auth", TestStatusKind::Failed)
                    .default_open(true)
                    .trigger(TestSuiteName::from_context())
                    .content(TestSuiteContent::new(Vec::<AnyElement>::new()))
                    .into_element(cx)
            });

        assert!(find_text_by_content(&element, "Auth").is_some());
    }

    #[test]
    fn test_row_provides_context_to_default_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Test::new("unit::auth::login", TestStatusKind::Passed)
                    .duration_ms(42)
                    .into_element(cx)
            });

        assert!(find_text_by_content(&element, "unit::auth::login").is_some());
        assert!(find_text_by_content(&element, "42ms").is_some());
    }
}
