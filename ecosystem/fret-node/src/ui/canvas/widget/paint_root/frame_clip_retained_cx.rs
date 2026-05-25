use crate::ui::canvas::widget::*;

impl<H: UiHost> super::frame_clip_adapter::PaintRootFrameClipCx<H> for PaintCx<'_, H> {
    fn push_paint_root_frame_clip_rect(&mut self, rect: Rect) {
        self.scene.push(SceneOp::PushClipRect { rect });
    }
}
