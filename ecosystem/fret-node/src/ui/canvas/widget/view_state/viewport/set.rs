use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn set_viewport_with_options<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        pan: CanvasPoint,
        zoom: f32,
        options: Option<&crate::ui::view_queue::NodeGraphViewQueueSetViewportOptions>,
    ) -> bool {
        let snapshot = self.sync_view_state(host);

        let mut target_min_zoom = self.style.geometry.min_zoom;
        let mut target_max_zoom = self.style.geometry.max_zoom;
        if let Some(options) = options {
            if let Some(min) = options.min_zoom
                && min.is_finite()
                && min > 0.0
            {
                target_min_zoom = target_min_zoom.max(min);
            }
            if let Some(max) = options.max_zoom
                && max.is_finite()
                && max > 0.0
            {
                target_max_zoom = target_max_zoom.min(max);
            }
        }
        if !target_min_zoom.is_finite()
            || !target_max_zoom.is_finite()
            || target_min_zoom <= 0.0
            || target_max_zoom <= 0.0
            || target_min_zoom > target_max_zoom
        {
            target_min_zoom = self.style.geometry.min_zoom;
            target_max_zoom = self.style.geometry.max_zoom;
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
}
