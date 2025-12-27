use fret_core::{Axis, Point, Px, Rect, Size};
use fret_ui::{LayoutCx, PaintCx, UiHost, Widget};

pub struct Split {
    pub axis: Axis,
    pub fraction: f32,
}

impl Split {
    pub fn new(axis: Axis, fraction: f32) -> Self {
        Self { axis, fraction }
    }
}

impl<H: UiHost> Widget<H> for Split {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let Some((&a, rest)) = cx.children.split_first() else {
            return Size::default();
        };
        let b = rest.first().copied();

        let f = self.fraction.clamp(0.0, 1.0);
        match self.axis {
            Axis::Horizontal => {
                let w0 = Px(cx.available.width.0 * f);
                let w1 = Px((cx.available.width.0 - w0.0).max(0.0));
                let rect_a = Rect::new(cx.bounds.origin, Size::new(w0, cx.available.height));
                let _ = cx.layout_in(a, rect_a);
                if let Some(b) = b {
                    let origin_b = Point::new(Px(cx.bounds.origin.x.0 + w0.0), cx.bounds.origin.y);
                    let rect_b = Rect::new(origin_b, Size::new(w1, cx.available.height));
                    let _ = cx.layout_in(b, rect_b);
                }
            }
            Axis::Vertical => {
                let h0 = Px(cx.available.height.0 * f);
                let h1 = Px((cx.available.height.0 - h0.0).max(0.0));
                let rect_a = Rect::new(cx.bounds.origin, Size::new(cx.available.width, h0));
                let _ = cx.layout_in(a, rect_a);
                if let Some(b) = b {
                    let origin_b = Point::new(cx.bounds.origin.x, Px(cx.bounds.origin.y.0 + h0.0));
                    let rect_b = Rect::new(origin_b, Size::new(cx.available.width, h1));
                    let _ = cx.layout_in(b, rect_b);
                }
            }
        }

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        let Some((&a, rest)) = cx.children.split_first() else {
            return;
        };
        let b = rest.first().copied();

        let f = self.fraction.clamp(0.0, 1.0);
        match self.axis {
            Axis::Horizontal => {
                let w0 = Px(cx.bounds.size.width.0 * f);
                let w1 = Px((cx.bounds.size.width.0 - w0.0).max(0.0));
                let rect_a = Rect::new(cx.bounds.origin, Size::new(w0, cx.bounds.size.height));
                cx.paint(a, rect_a);
                if let Some(b) = b {
                    let origin_b = Point::new(Px(cx.bounds.origin.x.0 + w0.0), cx.bounds.origin.y);
                    let rect_b = Rect::new(origin_b, Size::new(w1, cx.bounds.size.height));
                    cx.paint(b, rect_b);
                }
            }
            Axis::Vertical => {
                let h0 = Px(cx.bounds.size.height.0 * f);
                let h1 = Px((cx.bounds.size.height.0 - h0.0).max(0.0));
                let rect_a = Rect::new(cx.bounds.origin, Size::new(cx.bounds.size.width, h0));
                cx.paint(a, rect_a);
                if let Some(b) = b {
                    let origin_b = Point::new(cx.bounds.origin.x, Px(cx.bounds.origin.y.0 + h0.0));
                    let rect_b = Rect::new(origin_b, Size::new(cx.bounds.size.width, h1));
                    cx.paint(b, rect_b);
                }
            }
        }
    }
}
