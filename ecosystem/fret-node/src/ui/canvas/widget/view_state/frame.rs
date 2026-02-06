use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn frame_nodes_in_view<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        bounds: Rect,
        node_ids: &[GraphNodeId],
    ) -> bool {
        self.frame_nodes_in_view_with_options(host, window, bounds, node_ids, None)
    }

    pub(in super::super) fn frame_nodes_in_view_with_options<H: UiHost>(
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
