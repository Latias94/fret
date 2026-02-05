use std::sync::Arc;

use fret_core::{FontWeight, SemanticsRole, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};

use fret_ui_shadcn::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent,
    CollapsibleTrigger, Spinner,
};

use crate::elements::MessageResponse;
use crate::model::{ToolCall, ToolCallPayload, ToolCallState};

fn state_badge_label(state: ToolCallState) -> &'static str {
    match state {
        ToolCallState::Pending => "Pending",
        ToolCallState::Running => "Running",
        ToolCallState::Succeeded => "Succeeded",
        ToolCallState::Failed => "Failed",
        ToolCallState::Cancelled => "Cancelled",
    }
}

fn state_badge_variant(state: ToolCallState) -> BadgeVariant {
    match state {
        ToolCallState::Pending => BadgeVariant::Secondary,
        ToolCallState::Running => BadgeVariant::Secondary,
        ToolCallState::Succeeded => BadgeVariant::Default,
        ToolCallState::Failed => BadgeVariant::Destructive,
        ToolCallState::Cancelled => BadgeVariant::Outline,
    }
}

fn payload_to_markdown(payload: &ToolCallPayload) -> Arc<str> {
    match payload {
        ToolCallPayload::Text(text) => Arc::from(format!("```text\n{text}\n```")),
        ToolCallPayload::Json(value) => {
            let pretty = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
            Arc::from(format!("```json\n{pretty}\n```"))
        }
    }
}

#[derive(Clone)]
pub struct ToolCallBlock {
    call: ToolCall,
    default_open: bool,
    test_id_root: Option<Arc<str>>,
    test_id_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for ToolCallBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolCallBlock")
            .field("id", &self.call.id.as_ref())
            .field("name", &self.call.name.as_ref())
            .field("state", &self.call.state)
            .field("default_open", &self.default_open)
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_trigger", &self.test_id_trigger.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl ToolCallBlock {
    pub fn new(call: ToolCall) -> Self {
        Self {
            call,
            default_open: false,
            test_id_root: None,
            test_id_trigger: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_trigger(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(id.into());
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

        let call = self.call;
        let state = call.state;

        let header = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            |cx| {
                let mut left = Vec::new();

                if state == ToolCallState::Running {
                    left.push(
                        Spinner::new()
                            .refine_layout(
                                LayoutRefinement::default()
                                    .w_px(fret_core::Px(14.0))
                                    .h_px(fret_core::Px(14.0)),
                            )
                            .into_element(cx),
                    );
                }

                let name_style = TextStyle {
                    font: Default::default(),
                    size: theme.metric_required("font.size_sm"),
                    weight: FontWeight::MEDIUM,
                    slant: Default::default(),
                    line_height: Some(theme.metric_required("font.line_height")),
                    letter_spacing_em: None,
                };

                left.push(cx.text_props(TextProps {
                    layout: fret_ui::element::LayoutStyle::default(),
                    text: call.name.clone(),
                    style: Some(name_style),
                    color: None,
                    wrap: TextWrap::Word,
                    overflow: fret_core::TextOverflow::Clip,
                }));

                left.push(
                    Badge::new(state_badge_label(state))
                        .variant(state_badge_variant(state))
                        .into_element(cx),
                );

                left
            },
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |cx| {
                let mut out = Vec::new();

                if let Some(input) = call.input.as_ref() {
                    out.push(cx.text("Input"));
                    out.push(MessageResponse::new(payload_to_markdown(input)).into_element(cx));
                }

                if let Some(output) = call.output.as_ref() {
                    out.push(cx.text("Output"));
                    out.push(MessageResponse::new(payload_to_markdown(output)).into_element(cx));
                }

                if let Some(error) = call.error.as_ref() {
                    out.push(cx.text_props(TextProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        text: Arc::<str>::from("Error"),
                        style: None,
                        color: Some(theme.color_required("destructive")),
                        wrap: fret_core::TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                    }));
                    out.push(cx.text_props(TextProps {
                        layout: fret_ui::element::LayoutStyle::default(),
                        text: error.clone(),
                        style: None,
                        color: Some(theme.color_required("destructive")),
                        wrap: TextWrap::Word,
                        overflow: fret_core::TextOverflow::Clip,
                    }));
                }

                if out.is_empty() {
                    out.push(cx.text("No details."));
                }

                out
            },
        );

        let trigger_test_id = self.test_id_trigger;
        let root_test_id = self.test_id_root;

        let collapsible = Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout.w_full())
            .refine_style(self.chrome)
            .into_element_with_open_model(
                cx,
                move |cx, open, is_open| {
                    let chevron = Button::new(if is_open { "Hide" } else { "Show" })
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::Sm)
                        .into_element(cx);

                    let trigger_row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N2),
                        |_cx| vec![header, chevron],
                    );

                    let trigger = CollapsibleTrigger::new(open, vec![trigger_row])
                        .a11y_label("Toggle tool call details")
                        .into_element(cx, is_open);

                    let Some(test_id) = trigger_test_id.clone() else {
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
                },
                move |cx| {
                    CollapsibleContent::new(vec![content])
                        .refine_layout(LayoutRefinement::default().w_full())
                        .refine_style(ChromeRefinement::default().pt(Space::N2))
                        .into_element(cx)
                },
            );

        let Some(test_id) = root_test_id else {
            return collapsible;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![collapsible],
        )
    }
}
