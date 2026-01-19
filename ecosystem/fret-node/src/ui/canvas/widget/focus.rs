use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn repair_focused_edge_after_graph_change<H: UiHost>(
        &mut self,
        host: &mut H,
        preferred: Option<EdgeId>,
    ) {
        if preferred.is_none() && self.interaction.focused_edge.is_none() {
            return;
        }

        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.edges_focusable && !snapshot.interaction.edges_reconnectable {
            self.interaction.focused_edge = None;
            return;
        }

        let (edges, current_valid) = self
            .graph
            .read_ref(host, |g| {
                let mut edges: Vec<EdgeId> = g.edges.keys().copied().collect();
                edges.sort_unstable();
                let current = self.interaction.focused_edge;
                let current_valid = current.is_some_and(|id| g.edges.contains_key(&id));
                (edges, current_valid)
            })
            .ok()
            .unwrap_or_default();

        if edges.is_empty() {
            self.interaction.focused_edge = None;
            return;
        }

        if current_valid {
            return;
        }

        let base = preferred.or(self.interaction.focused_edge);
        let next = match base {
            Some(id) => match edges.binary_search(&id) {
                Ok(ix) => edges.get(ix).copied(),
                Err(ix) => edges.get(ix).copied().or_else(|| edges.first().copied()),
            },
            None => edges.first().copied(),
        };
        self.interaction.focused_edge = next;
    }

    pub(super) fn draw_order_hash(ids: &[GraphNodeId]) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        ids.hash(&mut hasher);
        hasher.finish()
    }
}
