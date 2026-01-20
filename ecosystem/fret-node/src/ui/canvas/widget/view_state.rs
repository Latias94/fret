use super::super::state::ViewportAnimationInterpolate;
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

    pub(super) fn frame_nodes_in_view<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        node_ids: &[GraphNodeId],
    ) -> bool {
        if node_ids.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no selection to frame",
            );
            return false;
        }

        #[derive(Debug, Clone, Copy)]
        struct NodeInfo {
            pos: CanvasPoint,
            w: f32,
            h: f32,
        }

        let infos: Vec<NodeInfo> = self
            .graph
            .read_ref(host, |graph| {
                let mut out: Vec<NodeInfo> = Vec::new();
                for id in node_ids {
                    let Some(node) = graph.nodes.get(id) else {
                        continue;
                    };
                    let (inputs, outputs) = node_ports(graph, *id);
                    let (w, h) = self.node_default_size_for_ports(inputs.len(), outputs.len());
                    out.push(NodeInfo {
                        pos: node.pos,
                        w,
                        h,
                    });
                }
                out
            })
            .ok()
            .unwrap_or_default();

        if infos.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no selection to frame",
            );
            return false;
        }

        let viewport_w = bounds.size.width.0;
        let viewport_h = bounds.size.height.0;
        if !viewport_w.is_finite()
            || !viewport_h.is_finite()
            || viewport_w <= 1.0
            || viewport_h <= 1.0
        {
            return false;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_w = 0.0f32;
        let mut max_h = 0.0f32;
        for n in &infos {
            min_x = min_x.min(n.pos.x);
            min_y = min_y.min(n.pos.y);
            max_x = max_x.max(n.pos.x);
            max_y = max_y.max(n.pos.y);
            max_w = max_w.max(n.w);
            max_h = max_h.max(n.h);
        }

        let spread_x = (max_x - min_x).max(0.0);
        let spread_y = (max_y - min_y).max(0.0);

        let margin = 48.0f32;
        let mut zoom_x = self.style.max_zoom;
        let mut zoom_y = self.style.max_zoom;
        if spread_x > 1.0e-3 {
            zoom_x = (viewport_w - max_w - 2.0 * margin) / spread_x;
        }
        if spread_y > 1.0e-3 {
            zoom_y = (viewport_h - max_h - 2.0 * margin) / spread_y;
        }

        let mut zoom = zoom_x.min(zoom_y);
        if !zoom.is_finite() {
            zoom = 1.0;
        }
        zoom = zoom.clamp(self.style.min_zoom, self.style.max_zoom);

        let mut rect_min_x = f32::INFINITY;
        let mut rect_min_y = f32::INFINITY;
        let mut rect_max_x = f32::NEG_INFINITY;
        let mut rect_max_y = f32::NEG_INFINITY;
        for n in &infos {
            let w = n.w / zoom;
            let h = n.h / zoom;
            rect_min_x = rect_min_x.min(n.pos.x);
            rect_min_y = rect_min_y.min(n.pos.y);
            rect_max_x = rect_max_x.max(n.pos.x + w);
            rect_max_y = rect_max_y.max(n.pos.y + h);
        }

        if !rect_min_x.is_finite()
            || !rect_min_y.is_finite()
            || !rect_max_x.is_finite()
            || !rect_max_y.is_finite()
        {
            return false;
        }

        let center_x = 0.5 * (rect_min_x + rect_max_x);
        let center_y = 0.5 * (rect_min_y + rect_max_y);

        let viewport_w_canvas = viewport_w / zoom;
        let viewport_h_canvas = viewport_h / zoom;
        let target_center_x = 0.5 * viewport_w_canvas;
        let target_center_y = 0.5 * viewport_h_canvas;

        let new_pan = CanvasPoint {
            x: target_center_x - center_x,
            y: target_center_y - center_y,
        };

        let snapshot = self.sync_view_state(host);
        let duration_ms = snapshot.interaction.frame_view_duration_ms;
        let duration = std::time::Duration::from_millis(duration_ms as u64);
        let interpolate = match snapshot.interaction.frame_view_interpolate {
            crate::io::NodeGraphViewportInterpolate::Linear => ViewportAnimationInterpolate::Linear,
            crate::io::NodeGraphViewportInterpolate::Smooth => ViewportAnimationInterpolate::Smooth,
        };

        let dx = new_pan.x - snapshot.pan.x;
        let dy = new_pan.y - snapshot.pan.y;
        let dzoom = zoom - snapshot.zoom;
        let needs_move = dx * dx + dy * dy > 1.0e-6 || dzoom.abs() > 1.0e-6;

        if duration.is_zero() || !needs_move {
            self.stop_viewport_animation_timer(host);
            self.update_view_state(host, |s| {
                s.zoom = zoom;
                s.pan = new_pan;
            });
        } else {
            self.start_viewport_animation_to(
                host,
                window,
                snapshot.pan,
                snapshot.zoom,
                new_pan,
                zoom,
                duration,
                interpolate,
            );
        }

        true
    }
}
