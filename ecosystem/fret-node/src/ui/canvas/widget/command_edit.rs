use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_copy<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.copy_selection_to_clipboard(
            cx.app,
            cx.window,
            &snapshot.selected_nodes,
            &snapshot.selected_groups,
        );
        true
    }

    pub(super) fn cmd_cut<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_edit_remove::cmd_cut(self, cx, snapshot)
    }

    pub(super) fn cmd_paste<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let at = self.next_paste_canvas_point(bounds, snapshot);
        self.request_paste_at_canvas(cx.app, cx.window, at);
        true
    }

    pub(super) fn cmd_duplicate<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.duplicate_selection(
            cx.app,
            cx.window,
            &snapshot.selected_nodes,
            &snapshot.selected_groups,
        );
        super::command_ui::finish_command_paint(cx)
    }

    pub(super) fn cmd_delete_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::command_edit_remove::cmd_delete_selection(self, cx, snapshot)
    }
}
