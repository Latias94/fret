use fret_components_ui::LayoutRefinement;
use fret_components_ui::declarative::scroll;
use fret_ui::element::AnyElement;
use fret_ui::{ElementCx, UiHost};

#[derive(Debug, Clone)]
pub struct ScrollArea {
    children: Vec<AnyElement>,
    show_scrollbar: bool,
    layout: LayoutRefinement,
}

impl ScrollArea {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            children,
            show_scrollbar: true,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn show_scrollbar(mut self, show: bool) -> Self {
        self.show_scrollbar = show;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let children = self.children;
        scroll::overflow_scroll(cx, self.layout, self.show_scrollbar, move |_cx| children)
    }
}

pub fn scroll_area<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    ScrollArea::new(f(cx)).into_element(cx)
}
