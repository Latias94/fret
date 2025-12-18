use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, SceneOp, Size};

pub struct FixedPanel {
    pub height: Px,
    pub background: Color,
}

impl FixedPanel {
    pub fn new(height: Px, background: Color) -> Self {
        Self { height, background }
    }
}

impl Widget for FixedPanel {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        Size::new(cx.available.width, self.height)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(6.0)),
        });
    }
}
