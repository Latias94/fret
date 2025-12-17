use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{Point, Px, Rect, Size};

pub struct Column {
    pub spacing: Px,
    pub padding: Px,
}

impl Column {
    pub fn new() -> Self {
        Self {
            spacing: Px(0.0),
            padding: Px(0.0),
        }
    }

    pub fn with_spacing(mut self, spacing: Px) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn with_padding(mut self, padding: Px) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Column {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let pad = self.padding.0.max(0.0);
        let inner_origin = Point::new(Px(cx.bounds.origin.x.0 + pad), Px(cx.bounds.origin.y.0 + pad));
        let inner_width = Px((cx.available.width.0 - pad * 2.0).max(0.0));

        let mut y = inner_origin.y;
        let mut content_height = Px(0.0);

        for (i, &child) in cx.children.iter().enumerate() {
            if i > 0 {
                y = Px(y.0 + self.spacing.0.max(0.0));
                content_height = Px(content_height.0 + self.spacing.0.max(0.0));
            }

            let child_origin = Point::new(inner_origin.x, y);
            let child_bounds = Rect::new(child_origin, Size::new(inner_width, Px(1.0e9)));
            let child_size = cx.layout_in(child, child_bounds);
            let child_h = child_size.height;

            let final_bounds = Rect::new(child_origin, Size::new(inner_width, child_h));
            let _ = cx.layout_in(child, final_bounds);

            y = Px(y.0 + child_h.0);
            content_height = Px(content_height.0 + child_h.0);
        }

        let total_h = Px(content_height.0 + pad * 2.0);
        Size::new(cx.available.width, total_h)
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
