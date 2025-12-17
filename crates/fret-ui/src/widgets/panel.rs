use crate::widget::{LayoutCx, PaintCx, Widget};
use fret_core::{Color, Corners, DrawOrder, Edges, Px, SceneOp, Size};

pub struct ColoredPanel {
    pub background: Color,
    pub border: Edges,
    pub border_color: Color,
    pub corner_radii: Corners,
}

impl ColoredPanel {
    pub fn new(background: Color) -> Self {
        Self {
            background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        }
    }
}

impl Widget for ColoredPanel {
    fn layout(&mut self, cx: &mut LayoutCx<'_>) -> Size {
        cx.available
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: self.background,
            border: self.border,
            border_color: self.border_color,
            corner_radii: self.corner_radii,
        });
    }
}
