use super::*;

struct TwoViewportRects {
    a: Rect,
    b: Rect,
}

impl TwoViewportRects {
    fn new(a: Rect, b: Rect) -> Self {
        Self { a, b }
    }
}

impl<H: UiHost> Widget<H> for TwoViewportRects {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        if let Some(&a) = cx.children.first() {
            let _ = cx.layout_viewport_root(a, self.a);
        }
        if let Some(&b) = cx.children.get(1) {
            let _ = cx.layout_viewport_root(b, self.b);
        }
        cx.available
    }
}


mod basics;
mod container;
mod interactivity;
mod layout_engine;
mod scroll;
mod text;
mod viewport_roots;
