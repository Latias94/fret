use std::sync::Arc;

use fret_core::{FontWeight, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{Justify, LayoutRefinement, Space};

use fret_ui_shadcn::Card;

#[derive(Clone)]
/// A centered empty-state card for transcript/conversation surfaces.
pub struct ConversationEmptyState {
    title: Arc<str>,
    description: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ConversationEmptyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConversationEmptyState")
            .field("title", &self.title.as_ref())
            .field("has_description", &self.description.is_some())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ConversationEmptyState {
    pub fn new(title: impl Into<Arc<str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let title_style = TextStyle {
            font: Default::default(),
            size: theme.metric_required("font.size"),
            weight: FontWeight::MEDIUM,
            slant: Default::default(),
            line_height: Some(theme.metric_required("font.line_height")),
            letter_spacing_em: None,
        };

        let title = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.title,
            style: Some(title_style),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let description = self.description.map(|text| {
            cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text,
                style: None,
                color: theme.color_by_key("muted-foreground"),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            })
        });

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2),
            |_cx| {
                let mut out = Vec::new();
                out.push(title);
                if let Some(description) = description {
                    out.push(description);
                }
                out
            },
        );

        let centered = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .justify(Justify::Center)
                .gap(Space::N4),
            move |_cx| vec![body],
        );

        let card = Card::new(vec![centered])
            .refine_layout(
                LayoutRefinement::default()
                    .w_full()
                    .h_full()
                    .min_h(theme.metric_required("metric.size.lg")),
            )
            .refine_layout(self.layout)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return card;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![card],
        )
    }
}
