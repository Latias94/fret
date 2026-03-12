use super::super::*;

fn graph_node_ids(graph: &Graph) -> Vec<GraphNodeId> {
    graph.nodes.keys().copied().collect()
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn cmd_frame_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &snapshot.selected_nodes);
        super::super::command_ui::finish_command_paint_if(cx, did)
    }

    pub(in super::super) fn cmd_frame_all<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        _snapshot: &ViewSnapshot,
    ) -> bool {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let nodes = self
            .graph
            .read_ref(cx.app, graph_node_ids)
            .ok()
            .unwrap_or_default();
        let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &nodes);
        super::super::command_ui::finish_command_paint_if(cx, did)
    }
}

#[cfg(test)]
mod tests;
