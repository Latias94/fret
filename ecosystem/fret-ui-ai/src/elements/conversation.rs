use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{LayoutRefinement, Space};

/// A simple vertical conversation container.
#[derive(Debug, Clone)]
pub struct Conversation {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl Conversation {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let layout = self.layout;
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N3)
                .layout(layout.merge(LayoutRefinement::default().w_full())),
            move |_cx| children,
        )
    }
}
