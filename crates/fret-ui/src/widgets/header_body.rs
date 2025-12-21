use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{Point, Px, Rect, Size};

pub struct HeaderBody {
    pub header_height: Px,
}

impl HeaderBody {
    pub fn new(header_height: Px) -> Self {
        Self { header_height }
    }
}

impl Widget for HeaderBody {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let header_h = Px(self.header_height.0.max(0.0).min(cx.available.height.0));

        let header_bounds = Rect::new(cx.bounds.origin, Size::new(cx.available.width, header_h));
        let body_origin = Point::new(cx.bounds.origin.x, Px(cx.bounds.origin.y.0 + header_h.0));
        let body_h = Px((cx.available.height.0 - header_h.0).max(0.0));
        let body_bounds = Rect::new(body_origin, Size::new(cx.available.width, body_h));

        if let Some(&header) = cx.children.first() {
            let _ = cx.layout_in(header, header_bounds);
        }
        if cx.children.len() >= 2 {
            let body = cx.children[1];
            let _ = cx.layout_in(body, body_bounds);
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
