use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn set_viewport_with_options<H: UiHost>(
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

    pub(in super::super) fn update_view_state<H: UiHost>(
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

    pub(in super::super) fn ensure_canvas_point_visible<H: UiHost>(
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

        let viewport = Self::viewport_from_pan_zoom(bounds, snapshot.pan, zoom);
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
}
