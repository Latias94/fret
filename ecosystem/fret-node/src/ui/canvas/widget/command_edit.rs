use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_copy<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.copy_selection_to_clipboard(
            cx.app,
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
        self.copy_selection_to_clipboard(
            cx.app,
            &snapshot.selected_nodes,
            &snapshot.selected_groups,
        );

        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_edges = snapshot.selected_edges.clone();
        let selected_groups = snapshot.selected_groups.clone();
        let remove_ops = self
            .graph
            .read_ref(cx.app, |graph| {
                Self::delete_selection_ops(
                    graph,
                    &snapshot.interaction,
                    &selected_nodes,
                    &selected_edges,
                    &selected_groups,
                )
            })
            .ok()
            .unwrap_or_default();
        if remove_ops.is_empty() {
            return true;
        }
        let (removed_nodes, removed_edges, removed_groups) =
            Self::removed_ids_from_ops(&remove_ops);
        let _ = self.commit_ops(cx.app, cx.window, Some("Cut"), remove_ops);
        self.update_view_state(cx.app, |s| {
            s.selected_edges.retain(|id| !removed_edges.contains(id));
            s.selected_nodes.retain(|id| !removed_nodes.contains(id));
            s.selected_groups.retain(|id| !removed_groups.contains(id));
        });

        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
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
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }

    pub(super) fn cmd_delete_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let preferred_focus = self
            .interaction
            .focused_edge
            .or_else(|| snapshot.selected_edges.first().copied());
        let selected_edges = snapshot.selected_edges.clone();
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_edges.is_empty() && selected_nodes.is_empty() && selected_groups.is_empty() {
            return true;
        }

        let remove_ops = self
            .graph
            .read_ref(cx.app, |graph| {
                Self::delete_selection_ops(
                    graph,
                    &snapshot.interaction,
                    &selected_nodes,
                    &selected_edges,
                    &selected_groups,
                )
            })
            .ok()
            .unwrap_or_default();

        if remove_ops.is_empty() {
            return true;
        }
        let (removed_nodes, removed_edges, removed_groups) =
            Self::removed_ids_from_ops(&remove_ops);
        let _ = self.commit_ops(cx.app, cx.window, Some("Delete Selection"), remove_ops);
        self.update_view_state(cx.app, |s| {
            s.selected_edges.retain(|id| !removed_edges.contains(id));
            s.selected_nodes.retain(|id| !removed_nodes.contains(id));
            s.selected_groups.retain(|id| !removed_groups.contains(id));
        });
        self.repair_focused_edge_after_graph_change(cx.app, preferred_focus);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }
}
