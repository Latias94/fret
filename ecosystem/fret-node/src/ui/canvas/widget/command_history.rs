use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_undo<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.undo_last(cx.app, cx.window);
        super::command_ui::finish_command_paint_if(cx, did)
    }

    pub(super) fn cmd_redo<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let did = self.redo_last(cx.app, cx.window);
        super::command_ui::finish_command_paint_if(cx, did)
    }
}
