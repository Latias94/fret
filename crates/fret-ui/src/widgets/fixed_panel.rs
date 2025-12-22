use crate::{
    UiHost,
    widget::{LayoutCx, PaintCx, Widget},
};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, Rect, SceneOp, Size};

pub struct FixedPanel {
    pub height: Px,
    pub background: Color,
}

impl FixedPanel {
    pub fn new(height: Px, background: Color) -> Self {
        Self { height, background }
    }
}

impl<H: UiHost> Widget<H> for FixedPanel {
    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        let size = Size::new(cx.available.width, self.height);
        let rect = Rect::new(cx.bounds.origin, size);
        if let Some(&child) = cx.children.first() {
            let _ = cx.layout_in(child, rect);
        }
        size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(6.0)),
        });

        if let Some(&child) = cx.children.first() {
            if let Some(bounds) = cx.child_bounds(child) {
                cx.paint(child, bounds);
            } else {
                cx.paint(child, cx.bounds);
            }
        }
    }
}
