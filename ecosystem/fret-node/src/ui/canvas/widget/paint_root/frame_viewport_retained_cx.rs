use crate::ui::canvas::widget::*;

impl<H: UiHost> super::frame_viewport_adapter::PaintRootFrameViewportCx<H> for PaintCx<'_, H> {
    fn paint_root_frame_bounds(&self) -> Rect {
        self.bounds
    }
}
