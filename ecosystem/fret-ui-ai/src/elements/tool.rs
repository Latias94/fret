use std::sync::Arc;

use fret_core::{SemanticsRole, TextOverflow, TextWrap};
use fret_icons::{IconId, ids};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{Badge, BadgeVariant, Collapsible, CollapsibleContent, CollapsibleTrigger};

use crate::elements::MessageResponse;
use crate::model::{ToolCallPayload, ToolCallState};

/// Tool disclosure building blocks aligned with AI Elements `tool.tsx`.
///
/// These components are ecosystem-layer policy surfaces:
/// - apps still own effects (IO/clipboard/open-url),
/// - components emit intents via action hooks (e.g. Collapsible triggers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl ToolStatus {
    pub fn from_tool_call_state(state: ToolCallState) -> Self {
        match state {
            ToolCallState::Pending => Self::Pending,
            ToolCallState::Running => Self::Running,
            ToolCallState::Succeeded => Self::Succeeded,
            ToolCallState::Failed => Self::Failed,
            ToolCallState::Cancelled => Self::Cancelled,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Succeeded => "Completed",
            Self::Failed => "Error",
            Self::Cancelled => "Cancelled",
        }
    }

    pub fn icon_id(self) -> IconId {
        match self {
            Self::Pending => ids::ui::STATUS_PENDING,
            Self::Running => ids::ui::STATUS_RUNNING,
            Self::Succeeded => ids::ui::STATUS_SUCCEEDED,
            Self::Failed => ids::ui::STATUS_FAILED,
            Self::Cancelled => ids::ui::STATUS_FAILED,
        }
    }

    pub fn badge_variant(self) -> BadgeVariant {
        match self {
            Self::Pending => BadgeVariant::Secondary,
            Self::Running => BadgeVariant::Secondary,
            Self::Succeeded => BadgeVariant::Secondary,
            Self::Failed => BadgeVariant::Secondary,
            Self::Cancelled => BadgeVariant::Secondary,
        }
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

        let label = self.title.unwrap_or_else(|| self.name.clone());
        let badge = Badge::new(self.status.label())
            .variant(self.status.badge_variant())
            .children([decl_icon::icon(cx, self.status.icon_id())])
            .into_element(cx);

        let left = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    decl_icon::icon(cx, ids::ui::TOOL),
                    cx.text(label.clone()),
                    badge,
                ]
            },
        );

        let chevron = decl_icon::icon(
            cx,
            if is_open {
                ids::ui::CHEVRON_UP
            } else {
                ids::ui::CHEVRON_DOWN
            },
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
        CollapsibleContent::new(self.children)
            .refine_layout(self.layout)
            .refine_style(ChromeRefinement::default().p(Space::N4).merge(self.chrome))
            .into_element(cx)
    }
}

fn payload_to_code_fence(payload: &ToolCallPayload) -> Arc<str> {
    match payload {
        ToolCallPayload::Text(text) => Arc::from(format!("```json\n{text}\n```")),
        ToolCallPayload::Json(value) => {
            let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
            Arc::from(format!("```json\n{pretty}\n```"))
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

        let code = MessageResponse::new(payload_to_code_fence(&self.input)).into_element(cx);
        let code = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded_md()
                    .bg(ColorRef::Token {
                        key: "muted",
                        fallback: ColorFallback::ThemeHoverBackground,
                    })
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
        let title = ToolSectionTitle::new(if self.error_text.is_some() {
            "Error"
        } else {
            "Result"
        })
        .into_element(cx);

        let mut body: Vec<AnyElement> = Vec::new();
        if let Some(error) = self.error_text.clone() {
            body.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: error,
                style: None,
                color: theme.color_by_key("destructive"),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            }));
        }
        if let Some(output) = self.output.as_ref() {
            body.push(MessageResponse::new(payload_to_code_fence(output)).into_element(cx));
        }

        let bg_key = if self.error_text.is_some() {
            "destructive"
        } else {
            "muted"
        };
        let container = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded_md()
                    .bg(ColorRef::Token {
                        key: bg_key,
                        fallback: ColorFallback::ThemeHoverBackground,
                    })
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
            layout: LayoutRefinement::default().w_full(),
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
        cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(fret_core::TextStyle {
                font: Default::default(),
                size: theme.metric_required("font.size"),
                weight: fret_core::FontWeight::MEDIUM,
                slant: Default::default(),
                line_height: Some(theme.metric_required("font.line_height")),
                letter_spacing_em: None,
            }),
            color: theme.color_by_key("muted-foreground"),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}
