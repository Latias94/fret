use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn next_paste_canvas_point(
        &mut self,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> CanvasPoint {
        clipboard_anchor::next_paste_canvas_point(self, bounds, snapshot)
    }

    pub(super) fn copy_selection_to_clipboard<H: UiHost>(
        &mut self,
        host: &mut H,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        clipboard_transfer::copy_selection_to_clipboard(self, host, selected_nodes, selected_groups)
    }

    pub(super) fn request_paste_at_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        clipboard_transfer::request_paste_at_canvas(self, host, window, at)
    }

    pub(super) fn apply_paste_text<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        text: &str,
        at: CanvasPoint,
    ) {
        clipboard_paste::apply_paste_text(self, host, window, text, at)
    }

    pub(super) fn duplicate_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        clipboard_paste::duplicate_selection(self, host, window, selected_nodes, selected_groups)
    }
}
