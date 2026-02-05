use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{AnyElement, SemanticsProps};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{Justify, LayoutRefinement, Space};

#[derive(Clone)]
pub struct MessageToolbar {
    children: Vec<AnyElement>,
    justify: Justify,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for MessageToolbar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageToolbar")
            .field("children_len", &self.children.len())
            .field("justify", &self.justify)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl MessageToolbar {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            justify: Justify::End,
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
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
        let layout = self.layout.merge(LayoutRefinement::default().w_full());
        let justify = self.justify;
        let children = self.children;

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(layout)
                .gap(Space::N2)
                .justify(justify),
            |_cx| children,
        );

        let Some(test_id) = self.test_id else {
            return row;
        };

        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| vec![row],
        )
    }
}
