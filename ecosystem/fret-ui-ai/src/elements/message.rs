use std::sync::Arc;

use fret_core::{FontWeight, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space};

/// Message role taxonomy aligned with typical chat UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

/// A minimal message bubble built on top of `fret-ui-shadcn::Card`.
#[derive(Debug, Clone)]
pub struct Message {
    role: MessageRole,
    text: Arc<str>,
    layout: LayoutRefinement,
}

impl Message {
    pub fn new(role: MessageRole, text: impl Into<Arc<str>>) -> Self {
        Self {
            role,
            text: text.into(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = self.text;

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

        let text_style = TextStyle {
            font: Default::default(),
            size: theme.metric_required("font.size"),
            weight: FontWeight::NORMAL,
            slant: Default::default(),
            line_height: Some(theme.metric_required("font.line_height")),
            letter_spacing_em: None,
        };

        fret_ui_shadcn::Card::new(vec![cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text,
            style: Some(text_style),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })])
        .refine_style(chrome)
        .refine_layout(self.layout)
        .into_element(cx)
    }
}
