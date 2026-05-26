use crate::ui::canvas::widget::*;

impl<H: UiHost> super::frame_background_adapter::PaintRootFrameBackgroundCx<H> for PaintCx<'_, H> {
    fn paint_root_frame_background(&mut self, viewport_rect: Rect, background: Color) {
        self.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: viewport_rect,
            background: fret_core::Paint::Solid(background).into(),
            border: Edges::all(Px(0.0)),
            border_paint: fret_core::Paint::TRANSPARENT.into(),
            corner_radii: Corners::all(Px(0.0)),
        });
    }
}
