use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
        self.sync_view_state_from_store_if_needed(host);

        let mut snapshot = ViewSnapshot {
            pan: self.cached_pan,
            zoom: self.cached_zoom,
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
            interaction: NodeGraphInteractionState::default(),
        };

        let _ = self.view_state.read(host, |_host, s| {
            snapshot.pan = s.pan;
            snapshot.zoom = s.zoom;
            snapshot.selected_nodes = s.selected_nodes.clone();
            snapshot.selected_edges = s.selected_edges.clone();
            snapshot.selected_groups = s.selected_groups.clone();
            snapshot.draw_order = s.draw_order.clone();
            snapshot.group_draw_order = s.group_draw_order.clone();
            snapshot.interaction = s.interaction.clone();
        });

        let zoom = snapshot.zoom;
        if zoom.is_finite() && zoom > 0.0 {
            self.cached_zoom = zoom.clamp(self.style.min_zoom, self.style.max_zoom);
        } else {
            self.cached_zoom = 1.0;
        }
        self.cached_pan = snapshot.pan;
        snapshot.zoom = self.cached_zoom;
        snapshot.pan = self.cached_pan;

        snapshot
    }

    pub(in super::super) fn sync_view_state_from_store_if_needed<H: UiHost>(
        &mut self,
        host: &mut H,
    ) {
        let Some(store) = self.store.as_ref() else {
            return;
        };
        let Some(rev) = store.revision(host) else {
            return;
        };
        if self.store_rev == Some(rev) {
            return;
        }
        self.store_rev = Some(rev);

        let Ok((next_view, next_graph)) =
            store.read_ref(host, |s| (s.view_state().clone(), s.graph().clone()))
        else {
            return;
        };
        let _ = self.graph.update(host, |g, _cx| {
            *g = next_graph;
        });
        let _ = self.view_state.update(host, |s, _cx| {
            *s = next_view;
        });
    }
}
