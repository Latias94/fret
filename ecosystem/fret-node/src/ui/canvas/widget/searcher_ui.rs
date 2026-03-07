use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn install_searcher_overlay(&mut self, searcher: SearcherState) {
        self.interaction.context_menu = None;
        self.interaction.searcher = Some(searcher);
    }

    pub(super) fn open_searcher_overlay(
        &mut self,
        invoked_at: Point,
        bounds: Rect,
        snapshot: &ViewSnapshot,
        target: ContextMenuTarget,
        candidates: Vec<InsertNodeCandidate>,
        rows_mode: SearcherRowsMode,
    ) {
        let searcher = build_searcher_state(
            self,
            invoked_at,
            bounds,
            snapshot,
            target,
            candidates,
            self.interaction.recent_kinds.clone(),
            rows_mode,
        );
        self.install_searcher_overlay(searcher);
    }

    pub(super) fn dismiss_searcher_overlay<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>) {
        self.interaction.searcher = None;
        self.interaction.pending_insert_node_drag = None;
        cx.release_pointer_capture();
    }
}

pub(super) fn invalidate_searcher_paint<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
}

pub(super) fn finish_searcher_event<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    cx.stop_propagation();
    invalidate_searcher_paint(cx);
    true
}
