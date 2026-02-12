use std::sync::Arc;

use fret_core::Color;
use fret_core::{FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_icons::{IconId, ids};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{Badge, BadgeVariant, Collapsible, CollapsibleContent, CollapsibleTrigger};

use crate::elements::CodeBlock;
use crate::model::{ToolCallPayload, ToolCallState};

/// Tool disclosure building blocks aligned with AI Elements `tool.tsx`.
///
/// These components are ecosystem-layer policy surfaces:
/// - apps still own effects (IO/clipboard/open-url),
/// - components emit intents via action hooks (e.g. Collapsible triggers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolStatus {
    ApprovalRequested,
    ApprovalResponded,
    InputAvailable,
    InputStreaming,
    OutputAvailable,
    OutputDenied,
    OutputError,
}

impl ToolStatus {
    pub fn from_tool_call_state(state: ToolCallState) -> Self {
        match state {
            ToolCallState::ApprovalRequested => Self::ApprovalRequested,
            ToolCallState::ApprovalResponded => Self::ApprovalResponded,
            ToolCallState::InputAvailable => Self::InputAvailable,
            ToolCallState::InputStreaming => Self::InputStreaming,
            ToolCallState::OutputAvailable => Self::OutputAvailable,
            ToolCallState::OutputDenied => Self::OutputDenied,
            ToolCallState::OutputError => Self::OutputError,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ApprovalRequested => "Awaiting Approval",
            Self::ApprovalResponded => "Responded",
            Self::InputAvailable => "Running",
            Self::InputStreaming => "Pending",
            Self::OutputAvailable => "Completed",
            Self::OutputDenied => "Denied",
            Self::OutputError => "Error",
        }
    }

    pub fn icon_id(self) -> IconId {
        match self {
            Self::ApprovalRequested => IconId::new_static("lucide.clock"),
            Self::ApprovalResponded => IconId::new_static("lucide.check-circle"),
            Self::InputAvailable => IconId::new_static("lucide.clock"),
            Self::InputStreaming => IconId::new_static("lucide.circle"),
            Self::OutputAvailable => IconId::new_static("lucide.check-circle"),
            Self::OutputDenied => IconId::new_static("lucide.x-circle"),
            Self::OutputError => IconId::new_static("lucide.x-circle"),
        }
    }

    pub fn badge_variant(self) -> BadgeVariant {
        match self {
            Self::ApprovalRequested => BadgeVariant::Secondary,
            Self::ApprovalResponded => BadgeVariant::Secondary,
            Self::InputAvailable => BadgeVariant::Secondary,
            Self::InputStreaming => BadgeVariant::Secondary,
            Self::OutputAvailable => BadgeVariant::Secondary,
            Self::OutputDenied => BadgeVariant::Secondary,
            Self::OutputError => BadgeVariant::Secondary,
        }
    }

    pub fn icon_color(self, theme: &Theme) -> Option<Color> {
        match self {
            // Align with AI Elements' Tailwind palette (600 shades).
            Self::ApprovalRequested => Some(Color {
                // yellow-600 (#ca8a04)
                r: 0.792,
                g: 0.541,
                b: 0.016,
                a: 1.0,
            }),
            Self::ApprovalResponded => Some(Color {
                // blue-600 (#2563eb)
                r: 0.145,
                g: 0.388,
                b: 0.922,
                a: 1.0,
            }),
            Self::OutputAvailable => Some(Color {
                // green-600 (#16a34a)
                r: 0.086,
                g: 0.639,
                b: 0.290,
                a: 1.0,
            }),
            Self::OutputDenied => Some(Color {
                // orange-600 (#ea580c)
                r: 0.918,
                g: 0.345,
                b: 0.047,
                a: 1.0,
            }),
            Self::OutputError => Some(theme.color_by_key("destructive").unwrap_or(Color {
                // red-600 (#dc2626)
                r: 0.863,
                g: 0.149,
                b: 0.149,
                a: 1.0,
            })),
            _ => None,
        }
    }
}

fn token_color_with_alpha(
    theme: &Theme,
    key: &'static str,
    fallback_key: &'static str,
    alpha: f32,
) -> Color {
    let base = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(fallback_key))
        .unwrap_or_else(|| theme.color_required("foreground"));
    let alpha = alpha.clamp(0.0, 1.0);
    Color {
        r: base.r,
        g: base.g,
        b: base.b,
        a: base.a * alpha,
    }
}

/// Tool disclosure header (Collapsible trigger row).
#[derive(Clone)]
pub struct ToolHeader {
    name: Arc<str>,
    title: Option<Arc<str>>,
    status: ToolStatus,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl ToolHeader {
    pub fn new(name: impl Into<Arc<str>>, status: ToolStatus) -> Self {
        Self {
            name: name.into(),
            title: None,
            status,
            test_id: None,
            layout: LayoutRefinement::default().w_full(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = Some(title.into());
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
        open_model: fret_runtime::Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let label = self.title.unwrap_or_else(|| derive_tool_label(&self.name));
        let status = self.status;
        let muted_foreground = theme.color_required("muted-foreground");

        let text_sm_px = theme
            .metric_by_key("component.text.sm_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let text_sm_line_height = theme
            .metric_by_key("component.text.sm_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));
        let label_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: label.clone(),
            style: Some(TextStyle {
                font: Default::default(),
                size: text_sm_px,
                weight: FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(text_sm_line_height),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let badge = Badge::new(self.status.label())
            .variant(self.status.badge_variant())
            .children([decl_icon::icon_with(
                cx,
                status.icon_id(),
                Some(Px(16.0)),
                status.icon_color(&theme).map(ColorRef::Color),
            )])
            .into_element(cx);

        let left = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    decl_icon::icon_with(
                        cx,
                        IconId::new_static("lucide.wrench"),
                        Some(Px(16.0)),
                        Some(ColorRef::Color(muted_foreground)),
                    ),
                    label_text,
                    badge,
                ]
            },
        );

        let chevron = decl_icon::icon_with(
            cx,
            if is_open {
                ids::ui::CHEVRON_UP
            } else {
                ids::ui::CHEVRON_DOWN
            },
            Some(Px(16.0)),
            Some(ColorRef::Color(muted_foreground)),
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .justify(Justify::Between)
                .items_center(),
            move |_cx| vec![left, chevron],
        );

        let trigger_row = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N3).merge(self.chrome),
                self.layout,
            ),
            move |_cx| vec![row],
        );

        let trigger = CollapsibleTrigger::new(open_model, vec![trigger_row])
            .a11y_label("Toggle tool details")
            .into_element(cx, is_open);

        let Some(test_id) = self.test_id else {
            return trigger;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Button,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![trigger],
        )
    }
}

impl std::fmt::Debug for ToolHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolHeader")
            .field("name", &self.name.as_ref())
            .field("title", &self.title.as_deref())
            .field("status", &self.status)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

/// Tool disclosure body wrapper (`CollapsibleContent`) with shadcn-like padding/gap defaults.
#[derive(Debug, Clone)]
pub struct ToolContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl ToolContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full(),
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

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N4),
            move |_cx| children,
        );

        CollapsibleContent::new([body])
            .refine_layout(self.layout)
            .refine_style(ChromeRefinement::default().p(Space::N4).merge(self.chrome))
            .into_element(cx)
    }
}

fn payload_to_jsonish_code(payload: &ToolCallPayload) -> Arc<str> {
    match payload {
        ToolCallPayload::Text(text) => text.clone(),
        ToolCallPayload::Json(value) => {
            let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
            Arc::from(pretty)
        }
    }
}

/// Tool call input section (AI Elements: “Parameters”).
#[derive(Debug, Clone)]
pub struct ToolInput {
    input: ToolCallPayload,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl ToolInput {
    pub fn new(input: ToolCallPayload) -> Self {
        Self {
            input,
            layout: LayoutRefinement::default().w_full(),
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let title = ToolSectionTitle::new("Parameters").into_element(cx);

        let code = CodeBlock::new(payload_to_jsonish_code(&self.input))
            .language("json")
            .show_header(false)
            .show_language(false)
            .into_element(cx);
        let bg = token_color_with_alpha(&theme, "muted", "accent", 0.5);
        let code = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded_md()
                    .bg(ColorRef::Color(bg))
                    .merge(self.chrome),
                self.layout,
            ),
            move |_cx| vec![code],
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            move |_cx| vec![title, code],
        )
    }
}

/// Tool call output section (AI Elements: “Result” / “Error”).
#[derive(Debug, Clone)]
pub struct ToolOutput {
    output: Option<ToolCallPayload>,
    error_text: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl ToolOutput {
    pub fn new(output: Option<ToolCallPayload>, error_text: Option<Arc<str>>) -> Self {
        Self {
            output,
            error_text,
            layout: LayoutRefinement::default().w_full(),
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

    pub fn into_element<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
    ) -> Option<AnyElement> {
        if self.output.is_none() && self.error_text.is_none() {
            return None;
        }

        let theme = Theme::global(&*cx.app).clone();

        let text_xs_px = theme
            .metric_by_key("component.text.xs_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let text_xs_line_height = theme
            .metric_by_key("component.text.xs_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));
        let title = ToolSectionTitle::new(if self.error_text.is_some() {
            "Error"
        } else {
            "Result"
        })
        .into_element(cx);

        let mut body: Vec<AnyElement> = Vec::new();
        if let Some(error) = self.error_text.clone() {
            body.push(
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: error,
                    style: Some(TextStyle {
                        font: Default::default(),
                        size: text_xs_px,
                        weight: FontWeight::NORMAL,
                        slant: Default::default(),
                        line_height: Some(text_xs_line_height),
                        letter_spacing_em: None,
                    }),
                    color: theme
                        .color_by_key("destructive")
                        .or_else(|| theme.color_by_key("foreground")),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                }),
            );
        }
        if let Some(output) = self.output.as_ref() {
            body.push(
                CodeBlock::new(payload_to_jsonish_code(output))
                    .language("json")
                    .show_header(false)
                    .show_language(false)
                    .into_element(cx),
            );
        }

        let bg = if self.error_text.is_some() {
            token_color_with_alpha(&theme, "destructive", "accent", 0.1)
        } else {
            token_color_with_alpha(&theme, "muted", "accent", 0.5)
        };
        let container = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded_md()
                    .bg(ColorRef::Color(bg))
                    .merge(self.chrome),
                self.layout,
            ),
            move |_cx| body,
        );

        Some(stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            move |_cx| vec![title, container],
        ))
    }
}

/// Tool disclosure root (bordered Collapsible).
#[derive(Debug, Clone)]
pub struct Tool {
    default_open: bool,
    header: ToolHeader,
    content: ToolContent,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Tool {
    pub fn new(header: ToolHeader, content: ToolContent) -> Self {
        Self {
            default_open: false,
            header,
            content,
            layout: LayoutRefinement::default().w_full().mb(Space::N4),
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
            .rounded_md()
            .border_1()
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

/// Small section title used by tool call surfaces.
#[derive(Debug, Clone)]
pub struct ToolSectionTitle {
    text: Arc<str>,
}

impl ToolSectionTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let text = Arc::<str>::from(self.text.to_ascii_uppercase());
        let text_xs_px = theme
            .metric_by_key("component.text.xs_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let text_xs_line_height = theme
            .metric_by_key("component.text.xs_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(fret_core::TextStyle {
                font: Default::default(),
                size: text_xs_px,
                weight: fret_core::FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(text_xs_line_height),
                // Tailwind `tracking-wide` is 0.025em.
                letter_spacing_em: Some(0.025),
            }),
            color: theme.color_by_key("muted-foreground"),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

fn derive_tool_label(name: &Arc<str>) -> Arc<str> {
    let raw = name.as_ref();
    raw.strip_prefix("tool-")
        .map(Arc::<str>::from)
        .unwrap_or_else(|| name.clone())
}
