use std::sync::Arc;

use fret_core::{FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use fret_ui_shadcn::Card;

use crate::elements::{MessageResponse, SourcesBlock, ToolCallBlock};
use crate::model::{MessagePart, MessageRole};

#[derive(Clone)]
pub struct MessageParts {
    role: MessageRole,
    parts: Arc<[MessagePart]>,
    on_link_activate: Option<fret_markdown::OnLinkActivate>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageParts")
            .field("role", &self.role)
            .field("parts_len", &self.parts.len())
            .field("has_on_link_activate", &self.on_link_activate.is_some())
            .field("layout", &self.layout)
            .finish()
    }
}

impl MessageParts {
    pub fn new(role: MessageRole, parts: impl Into<Arc<[MessagePart]>>) -> Self {
        Self {
            role,
            parts: parts.into(),
            on_link_activate: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn on_link_activate(mut self, on_link_activate: fret_markdown::OnLinkActivate) -> Self {
        self.on_link_activate = Some(on_link_activate);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let chrome = match self.role {
            MessageRole::User => {
                ChromeRefinement::default().bg(ColorRef::Color(theme.color_required("primary")))
            }
            MessageRole::Assistant => {
                ChromeRefinement::default().bg(ColorRef::Color(theme.color_required("card")))
            }
            MessageRole::System => {
                ChromeRefinement::default().bg(ColorRef::Color(theme.color_required("muted")))
            }
            MessageRole::Tool => {
                ChromeRefinement::default().bg(ColorRef::Color(theme.color_required("secondary")))
            }
        }
        .rounded(Radius::Lg)
        .p(Space::N4);

        let fg = match self.role {
            MessageRole::User => theme.color_required("primary-foreground"),
            _ => theme.color_required("foreground"),
        };

        let on_link_activate = self.on_link_activate;
        let parts = self.parts;

        let content = cx.stack(move |cx| {
            let mut out = Vec::new();
            for part in parts.iter() {
                match part {
                    MessagePart::Text(text) => {
                        let text_style = TextStyle {
                            font: Default::default(),
                            size: theme.metric_required("font.size"),
                            weight: FontWeight::NORMAL,
                            slant: Default::default(),
                            line_height: Some(theme.metric_required("font.line_height")),
                            letter_spacing_em: None,
                        };

                        out.push(cx.text_props(TextProps {
                            layout: LayoutStyle::default(),
                            text: text.clone(),
                            style: Some(text_style),
                            color: Some(fg),
                            wrap: TextWrap::Word,
                            overflow: TextOverflow::Clip,
                        }));
                    }
                    MessagePart::Markdown(source) => {
                        let mut response = MessageResponse::new(source.clone());
                        if let Some(handler) = on_link_activate.clone() {
                            response = response.on_link_activate(handler);
                        }
                        out.push(response.into_element(cx));
                    }
                    MessagePart::ToolCall(call) => {
                        out.push(ToolCallBlock::new(call.clone()).into_element(cx));
                    }
                    MessagePart::Sources(items) => {
                        let mut block = SourcesBlock::new(items.clone());
                        if let Some(handler) = on_link_activate.clone() {
                            block = block.on_open_url(handler);
                        }
                        out.push(block.into_element(cx));
                    }
                }
            }

            out
        });

        Card::new(vec![content])
            .refine_style(chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}
