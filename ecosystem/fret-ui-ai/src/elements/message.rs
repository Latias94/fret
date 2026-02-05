use std::sync::Arc;

use fret_core::{FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

use crate::MessageResponse;
use crate::model::MessageRole;

/// A minimal message bubble built on top of `fret-ui-shadcn::Card`.
#[derive(Debug, Clone)]
pub struct Message {
    role: MessageRole,
    body: MessageBody,
    layout: LayoutRefinement,
}

#[derive(Debug, Clone)]
enum MessageBody {
    Text(Arc<str>),
    Markdown(Arc<str>),
}

impl Message {
    pub fn new(role: MessageRole, text: impl Into<Arc<str>>) -> Self {
        Self {
            role,
            body: MessageBody::Text(text.into()),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn markdown(role: MessageRole, source: impl Into<Arc<str>>) -> Self {
        Self {
            role,
            body: MessageBody::Markdown(source.into()),
            layout: LayoutRefinement::default(),
        }
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

        let content = match self.body {
            MessageBody::Text(text) => {
                let text_style = TextStyle {
                    font: Default::default(),
                    size: theme.metric_required("font.size"),
                    weight: FontWeight::NORMAL,
                    slant: Default::default(),
                    line_height: Some(theme.metric_required("font.line_height")),
                    letter_spacing_em: None,
                };

                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text,
                    style: Some(text_style),
                    color: Some(fg),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                })
            }
            MessageBody::Markdown(source) => {
                // Markdown resolves its own text theme; keep the message chrome responsible for the
                // bubble background/padding and let markdown handle code fences, links, etc.
                //
                // Note: link activation is intentionally app-owned; callers should wrap this
                // element with a configured `MessageResponse` if they want `Effect::OpenUrl`.
                let response = MessageResponse::new(source).into_element(cx);
                return fret_ui_shadcn::Card::new(vec![response])
                    .refine_style(chrome)
                    .refine_layout(self.layout)
                    .into_element(cx);
            }
        };

        fret_ui_shadcn::Card::new(vec![content])
            .refine_style(chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}
