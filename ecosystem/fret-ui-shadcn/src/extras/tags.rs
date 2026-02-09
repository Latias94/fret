use std::sync::Arc;

use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::badge::{Badge, BadgeVariant};
use crate::test_id::attach_test_id;

/// A single, shadcn-styled static tag (chip).
#[derive(Debug, Clone)]
pub struct Tag {
    label: Arc<str>,
    variant: BadgeVariant,
    test_id: Option<Arc<str>>,
}

impl Tag {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::Secondary,
            test_id: None,
        }
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = Badge::new(self.label)
            .variant(self.variant)
            .into_element(cx);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.tag")),
        )
    }
}

/// A wrapping list of static tags.
#[derive(Debug, Clone)]
pub struct Tags {
    items: Vec<Tag>,
    variant: BadgeVariant,
    gap: Space,
    wrap: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Tags {
    pub fn new<I, S>(labels: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        let items = labels.into_iter().map(Tag::new).collect();
        Self {
            items,
            variant: BadgeVariant::Secondary,
            gap: Space::N2,
            wrap: true,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn items(mut self, items: impl IntoIterator<Item = Tag>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = decl_style::space(&theme, self.gap);
        let layout = decl_style::layout_style(&theme, self.layout);

        let variant = self.variant;
        let prefix = self
            .test_id
            .clone()
            .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.tags"));
        let wrap = self.wrap;
        let items = self.items;

        let el = cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Horizontal,
                gap,
                padding: fret_core::Edges::all(fret_core::Px(0.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap,
            },
            move |cx| {
                items
                    .into_iter()
                    .map(|tag| tag.variant(variant).into_element(cx))
                    .collect::<Vec<_>>()
            },
        );

        attach_test_id(el, prefix)
    }
}
