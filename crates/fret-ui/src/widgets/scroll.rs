use crate::widget::{EventCx, Invalidation, LayoutCx, PaintCx, Widget};
use fret_core::{Point, Px, Rect, Size, Event, PointerEvent, SceneOp};

pub struct Scroll {
    offset_y: Px,
}

impl Scroll {
    pub fn new() -> Self {
        Self { offset_y: Px(0.0) }
    }

    pub fn offset_y(&self) -> Px {
        self.offset_y
    }

    fn clamp_offset(&mut self, content_height: Px, viewport_height: Px) {
        let max = Px((content_height.0 - viewport_height.0).max(0.0));
        self.offset_y = Px(self.offset_y.0.clamp(0.0, max.0));
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Scroll {
    fn event(&mut self, cx: &mut EventCx<'_>, event: &Event) {
        let Event::Pointer(PointerEvent::Wheel { delta, .. }) = event else {
            return;
        };

        self.offset_y = Px((self.offset_y.0 - delta.y.0).max(0.0));
        cx.invalidate_self(Invalidation::Layout);
        cx.invalidate_self(Invalidation::Paint);
        cx.request_redraw();
        cx.stop_propagation();
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        let Some(&child) = cx.children.first() else {
            return cx.available;
        };

        // Measure content with unconstrained height (very simple MVP).
        let content_size = cx.layout_in(
            child,
            Rect::new(cx.bounds.origin, Size::new(cx.available.width, Px(1.0e9))),
        );

        self.clamp_offset(content_size.height, cx.available.height);

        // Layout child at a translated origin to implement scrolling.
        let origin = Point::new(cx.bounds.origin.x, Px(cx.bounds.origin.y.0 - self.offset_y.0));
        let child_bounds = Rect::new(origin, Size::new(cx.available.width, content_size.height));
        let _ = cx.layout_in(child, child_bounds);

        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        let Some(&child) = cx.children.first() else {
            return;
        };

        cx.scene.push(SceneOp::PushClipRect { rect: cx.bounds });

        if let Some(bounds) = cx.child_bounds(child) {
            cx.paint(child, bounds);
        } else {
            cx.paint(child, cx.bounds);
        }

        cx.scene.push(SceneOp::PopClip);
    }
}
