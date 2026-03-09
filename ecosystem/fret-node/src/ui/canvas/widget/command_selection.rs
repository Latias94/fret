use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_select_all<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return true;
        }
        let (nodes, groups, edges) = self
            .graph
            .read_ref(cx.app, |graph| {
                let nodes = graph
                    .nodes
                    .keys()
                    .copied()
                    .filter(|id| Self::node_is_selectable(graph, &snapshot.interaction, *id))
                    .collect::<Vec<_>>();
                let groups = graph.groups.keys().copied().collect::<Vec<_>>();
                let edges = if snapshot.interaction.edges_selectable {
                    graph
                        .edges
                        .keys()
                        .copied()
                        .filter(|id| Self::edge_is_selectable(graph, &snapshot.interaction, *id))
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
                (nodes, groups, edges)
            })
            .ok()
            .unwrap_or_default();

        self.interaction.focused_edge = None;
        self.interaction.focused_node = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(cx.app, |s| {
            s.selected_nodes = nodes;
            s.selected_groups = groups;
            s.selected_edges = edges;
        });
        super::command_ui::finish_command_paint(cx)
    }
}
