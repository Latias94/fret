use crate::ui::canvas::widget::*;

impl<H: UiHost> super::tail_cleanup_adapter::PaintRootTailCleanupCx<H> for PaintCx<'_, H> {
    fn pop_paint_root_tail_clip(&mut self) {
        self.scene.push(SceneOp::PopClip);
    }
}
