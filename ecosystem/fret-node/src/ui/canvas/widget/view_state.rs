use super::super::state::{ViewportAnimationEase, ViewportAnimationInterpolate};
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

    pub(super) fn drain_view_queue<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
    ) -> bool {
        let Some(queue) = self.view_queue.as_ref() else {
            return false;
        };
        let Some(rev) = queue.revision(host) else {
            return false;
        };
        if self.view_queue_key == Some(rev) {
            return false;
        }
        self.view_queue_key = Some(rev);

        let Ok(reqs) = queue.update(host, |q, _cx| q.drain()) else {
            return false;
        };
        if reqs.is_empty() {
            return false;
        }

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let mut did = false;
        for req in reqs {
            match req {
                crate::ui::NodeGraphViewRequest::FrameNodes { nodes, options } => {
                    did |= self.frame_nodes_in_view_with_options(
                        host,
                        window,
                        bounds,
                        &nodes,
                        Some(&options),
                    );
                }
                crate::ui::NodeGraphViewRequest::SetViewport { pan, zoom, options } => {
                    did |= self.set_viewport_with_options(host, window, pan, zoom, Some(&options));
                }
            }
        }
        did
    }

    pub(super) fn set_viewport_with_options<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        pan: CanvasPoint,
        zoom: f32,
        options: Option<&crate::ui::NodeGraphSetViewportOptions>,
    ) -> bool {
        let snapshot = self.sync_view_state(host);

        let mut target_min_zoom = self.style.min_zoom;
        let mut target_max_zoom = self.style.max_zoom;
        if let Some(options) = options {
            if let Some(min) = options.min_zoom {
                if min.is_finite() && min > 0.0 {
                    target_min_zoom = target_min_zoom.max(min);
                }
            }
            if let Some(max) = options.max_zoom {
                if max.is_finite() && max > 0.0 {
                    target_max_zoom = target_max_zoom.min(max);
                }
            }
        }
        if !target_min_zoom.is_finite()
            || !target_max_zoom.is_finite()
            || target_min_zoom <= 0.0
            || target_max_zoom <= 0.0
            || target_min_zoom > target_max_zoom
        {
            target_min_zoom = self.style.min_zoom;
            target_max_zoom = self.style.max_zoom;
        }

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom.clamp(target_min_zoom, target_max_zoom)
        } else {
            snapshot.zoom
        };
        let pan = if pan.x.is_finite() && pan.y.is_finite() {
            pan
        } else {
            snapshot.pan
        };

        let duration_ms = options.and_then(|o| o.duration_ms).unwrap_or(0);
        let duration = std::time::Duration::from_millis(duration_ms as u64);
        let interpolate = options
            .and_then(|o| o.interpolate)
            .unwrap_or(snapshot.interaction.frame_view_interpolate);
        let interpolate = match interpolate {
            crate::io::NodeGraphViewportInterpolate::Linear => ViewportAnimationInterpolate::Linear,
            crate::io::NodeGraphViewportInterpolate::Smooth => ViewportAnimationInterpolate::Smooth,
        };
        let ease = options
            .and_then(|o| o.ease)
            .or(snapshot.interaction.frame_view_ease)
            .map(|ease| match ease {
                crate::io::NodeGraphViewportEase::Linear => ViewportAnimationEase::Linear,
                crate::io::NodeGraphViewportEase::Smoothstep => ViewportAnimationEase::Smoothstep,
                crate::io::NodeGraphViewportEase::CubicInOut => ViewportAnimationEase::CubicInOut,
            });

        let dx = pan.x - snapshot.pan.x;
        let dy = pan.y - snapshot.pan.y;
        let dzoom = zoom - snapshot.zoom;
        let needs_move = dx * dx + dy * dy > 1.0e-6 || dzoom.abs() > 1.0e-6;
        if !needs_move {
            return false;
        }

        if duration.is_zero() {
            self.stop_viewport_animation_timer(host);
            self.update_view_state(host, |s| {
                s.pan = pan;
                s.zoom = zoom;
            });
        } else {
            self.start_viewport_animation_to(
                host,
                window,
                snapshot.pan,
                snapshot.zoom,
                pan,
                zoom,
                duration,
                interpolate,
                ease,
            );
        }

        true
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

        let viewport = CanvasViewport2D::new(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(snapshot.pan.x), Px(snapshot.pan.y)),
                zoom,
            },
        );
        let vis = viewport.visible_canvas_rect();
        let view_w = vis.size.width.0;
        let view_h = vis.size.height.0;

        let view_min_x = vis.origin.x.0;
        let view_min_y = vis.origin.y.0;

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
        self.frame_nodes_in_view_with_options(host, window, bounds, node_ids, None)
    }

    pub(super) fn frame_nodes_in_view_with_options<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        node_ids: &[GraphNodeId],
        options: Option<&crate::ui::NodeGraphFitViewOptions>,
    ) -> bool {
        let snapshot = self.sync_view_state(host);
        let include_hidden_nodes = options.is_some_and(|o| o.include_hidden_nodes);

        let mut target_min_zoom = self.style.min_zoom;
        let mut target_max_zoom = self.style.max_zoom;
        if let Some(options) = options {
            if let Some(min) = options.min_zoom {
                if min.is_finite() && min > 0.0 {
                    target_min_zoom = target_min_zoom.max(min);
                }
            }
            if let Some(max) = options.max_zoom {
                if max.is_finite() && max > 0.0 {
                    target_max_zoom = target_max_zoom.min(max);
                }
            }
        }
        if !target_min_zoom.is_finite()
            || !target_max_zoom.is_finite()
            || target_min_zoom <= 0.0
            || target_max_zoom <= 0.0
            || target_min_zoom > target_max_zoom
        {
            target_min_zoom = self.style.min_zoom;
            target_max_zoom = self.style.max_zoom;
        }

        if node_ids.is_empty() {
            self.show_toast(
                host,
                window,
                DiagnosticSeverity::Info,
                "no selection to frame",
            );
            return false;
        }

        let infos: Vec<crate::runtime::fit_view::FitViewNodeInfo> = self
            .graph
            .read_ref(host, |graph| {
                let mut out: Vec<crate::runtime::fit_view::FitViewNodeInfo> = Vec::new();
                for id in node_ids {
                    let Some(node) = graph.nodes.get(id) else {
                        continue;
                    };
                    if node.hidden && !include_hidden_nodes {
                        continue;
                    }
                    let (inputs, outputs) = node_ports(graph, *id);
                    let (w, h) = self.node_default_size_for_ports(inputs.len(), outputs.len());
                    out.push(crate::runtime::fit_view::FitViewNodeInfo {
                        pos: node.pos,
                        size_px: (w, h),
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

        let padding = options
            .and_then(|o| o.padding)
            .unwrap_or(snapshot.interaction.frame_view_padding);
        let padding = if padding.is_finite() {
            padding.clamp(0.0, 0.45)
        } else {
            0.0
        };
        let Some((new_pan, zoom)) = crate::runtime::fit_view::compute_fit_view_target(
            &infos,
            crate::runtime::fit_view::FitViewComputeOptions {
                viewport_width_px: viewport_w,
                viewport_height_px: viewport_h,
                node_origin: {
                    let origin = snapshot.interaction.node_origin.normalized();
                    (origin.x, origin.y)
                },
                padding,
                margin_px_fallback: 48.0,
                min_zoom: target_min_zoom,
                max_zoom: target_max_zoom,
            },
        ) else {
            return false;
        };

        let duration_ms = options
            .and_then(|o| o.duration_ms)
            .unwrap_or(snapshot.interaction.frame_view_duration_ms);
        let duration = std::time::Duration::from_millis(duration_ms as u64);
        let interpolate = options
            .and_then(|o| o.interpolate)
            .unwrap_or(snapshot.interaction.frame_view_interpolate);
        let interpolate = match interpolate {
            crate::io::NodeGraphViewportInterpolate::Linear => ViewportAnimationInterpolate::Linear,
            crate::io::NodeGraphViewportInterpolate::Smooth => ViewportAnimationInterpolate::Smooth,
        };
        let ease = options
            .and_then(|o| o.ease)
            .or(snapshot.interaction.frame_view_ease)
            .map(|ease| match ease {
                crate::io::NodeGraphViewportEase::Linear => ViewportAnimationEase::Linear,
                crate::io::NodeGraphViewportEase::Smoothstep => ViewportAnimationEase::Smoothstep,
                crate::io::NodeGraphViewportEase::CubicInOut => ViewportAnimationEase::CubicInOut,
            });

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
                ease,
            );
        }

        true
    }
}
