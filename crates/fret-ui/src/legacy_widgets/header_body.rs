use crate::{
    UiHost,
    widget::{LayoutCx, PaintCx, Widget},
};
use fret_core::{Point, Px, Rect, Size};

pub struct HeaderBody {
    pub header_height: Px,
}

impl HeaderBody {
    pub fn new(header_height: Px) -> Self {
        Self { header_height }
    }

    pub fn auto() -> Self {
        Self {
            header_height: Px(-1.0),
        }
    }

    fn is_auto(&self) -> bool {
        self.header_height.0 < 0.0
    }
}

impl<H: UiHost> Widget<H> for HeaderBody {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let mut header_h = Px(self.header_height.0.max(0.0).min(cx.available.height.0));

        if let Some(&header) = cx.children.first() {
            if self.is_auto() {
                let probe_bounds = Rect::new(
                    cx.bounds.origin,
                    Size::new(cx.available.width, cx.available.height),
                );
                let header_size = cx.layout_in(header, probe_bounds);
                header_h = Px(header_size.height.0.max(0.0).min(cx.available.height.0));
            }
        } else {
            header_h = Px(0.0);
        }

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

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        for &child in cx.children {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
