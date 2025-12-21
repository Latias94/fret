use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{Point, Px, Rect, Size};

pub struct Row {
    pub spacing: Px,
    pub padding: Px,
}

impl Row {
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

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Row {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let pad = self.padding.0.max(0.0);
        let spacing = self.spacing.0.max(0.0);

        let inner_origin = Point::new(
            Px(cx.bounds.origin.x.0 + pad),
            Px(cx.bounds.origin.y.0 + pad),
        );
        let inner_width = Px((cx.available.width.0 - pad * 2.0).max(0.0));

        let mut remaining_w = inner_width.0;
        let mut max_h = 0.0f32;

        let mut placements: Vec<(fret_core::NodeId, Point, Size)> = Vec::new();
        let mut x = inner_origin.x.0;

        for (i, &child) in cx.children.iter().enumerate() {
            if i > 0 {
                x += spacing;
                remaining_w = (remaining_w - spacing).max(0.0);
            }

            let is_last = i + 1 == cx.children.len();
            let probe_bounds = Rect::new(
                Point::new(Px(x), inner_origin.y),
                Size::new(Px(remaining_w), Px(1.0e9)),
            );
            let child_size = cx.layout_in(child, probe_bounds);

            let w = if is_last {
                Px(remaining_w)
            } else {
                Px(child_size.width.0.min(remaining_w))
            };
            let size = Size::new(w, child_size.height);
            placements.push((child, Point::new(Px(x), inner_origin.y), size));

            x += w.0;
            remaining_w = (remaining_w - w.0).max(0.0);
            max_h = max_h.max(child_size.height.0);
        }

        for (child, origin, size) in placements {
            let dy = (max_h - size.height.0).max(0.0) * 0.5;
            let child_origin = Point::new(origin.x, Px(origin.y.0 + dy));
            let bounds = Rect::new(child_origin, Size::new(size.width, Px(max_h)));
            let _ = cx.layout_in(child, bounds);
        }

        let total_h = if cx.children.is_empty() {
            Px(0.0)
        } else {
            Px(max_h + pad * 2.0)
        };

        let total_w = if cx.children.is_empty() {
            Px(0.0)
        } else {
            Px(inner_width.0 + pad * 2.0)
        };

        Size::new(Px(total_w.0.min(cx.available.width.0)), total_h)
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
