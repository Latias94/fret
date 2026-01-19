use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn sync_view_state<H: UiHost>(&mut self, host: &mut H) -> ViewSnapshot {
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

    pub(super) fn sync_view_state_from_store_if_needed<H: UiHost>(&mut self, host: &mut H) {
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

    pub(super) fn drain_edit_queue<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) {
        let Some(queue) = self.edit_queue.as_ref() else {
            return;
        };
        let Some(rev) = queue.revision(host) else {
            return;
        };
        if self.edit_queue_key == Some(rev) {
            return;
        }
        self.edit_queue_key = Some(rev);

        let Ok(txs) = queue.update(host, |q, _cx| q.drain()) else {
            return;
        };
        for tx in txs {
            let _ = self.commit_transaction(host, window, &tx);
        }
    }

    pub(super) fn update_view_state<H: UiHost>(
        &mut self,
        host: &mut H,
        f: impl FnOnce(&mut NodeGraphViewState),
    ) {
        let before = if self.callbacks.is_some() {
            if let Some(store) = self.store.as_ref() {
                store.read_ref(host, |s| s.view_state().clone()).ok()
            } else {
                self.view_state.read_ref(host, |s| s.clone()).ok()
            }
        } else {
            None
        };

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let style = self.style.clone();
        if let Some(store) = self.store.as_ref() {
            let _ = store.update(host, |store, _cx| {
                store.update_view_state(|s| {
                    f(s);

                    let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                        s.zoom.clamp(style.min_zoom, style.max_zoom)
                    } else {
                        1.0
                    };
                    s.zoom = zoom;

                    if let Some(extent) = s.interaction.translate_extent {
                        s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                    }
                });
            });
        } else {
            let _ = self.view_state.update(host, |s, _cx| {
                f(s);

                let zoom = if s.zoom.is_finite() && s.zoom > 0.0 {
                    s.zoom.clamp(style.min_zoom, style.max_zoom)
                } else {
                    1.0
                };
                s.zoom = zoom;

                if let Some(extent) = s.interaction.translate_extent {
                    s.pan = Self::clamp_pan_to_translate_extent(s.pan, zoom, bounds, extent);
                }
            });
        }
        self.sync_view_state(host);

        if let Some(before) = before {
            let after = self.view_state.read_ref(host, |s| s.clone()).ok();
            if let Some(after) = after {
                let mut changes: Vec<ViewChange> = Vec::new();
                if before.pan != after.pan || (before.zoom - after.zoom).abs() > 1.0e-6 {
                    changes.push(ViewChange::Viewport {
                        pan: after.pan,
                        zoom: after.zoom,
                    });
                }
                if before.selected_nodes != after.selected_nodes
                    || before.selected_edges != after.selected_edges
                    || before.selected_groups != after.selected_groups
                {
                    changes.push(ViewChange::Selection {
                        nodes: after.selected_nodes.clone(),
                        edges: after.selected_edges.clone(),
                        groups: after.selected_groups.clone(),
                    });
                }
                self.emit_view_callbacks(&changes);
            }
        }
    }

    pub(super) fn ensure_canvas_point_visible<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        point: CanvasPoint,
    ) {
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }
        if bounds.size.width.0 <= 0.0 || bounds.size.height.0 <= 0.0 {
            return;
        }

        let margin_screen = 24.0f32;
        let margin = margin_screen / zoom;
        if !margin.is_finite() {
            return;
        }

        let view_w = bounds.size.width.0 / zoom;
        let view_h = bounds.size.height.0 / zoom;

        let view_min_x = -snapshot.pan.x;
        let view_min_y = -snapshot.pan.y;

        let mut pan = snapshot.pan;

        let min_x = view_min_x + margin;
        let max_x = view_min_x + view_w - margin;
        if point.x < min_x {
            pan.x = margin - point.x;
        } else if point.x > max_x {
            pan.x = (view_w - margin) - point.x;
        }

        let min_y = view_min_y + margin;
        let max_y = view_min_y + view_h - margin;
        if point.y < min_y {
            pan.y = margin - point.y;
        } else if point.y > max_y {
            pan.y = (view_h - margin) - point.y;
        }

        if pan != snapshot.pan {
            self.update_view_state(host, |s| {
                s.pan = pan;
            });
        }
    }
}
