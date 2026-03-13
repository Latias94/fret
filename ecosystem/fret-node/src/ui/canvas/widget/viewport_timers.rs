use super::*;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn stop_viewport_animation_timer<H: UiHost>(&mut self, host: &mut H) {
        super::viewport_timer_animation::stop_viewport_animation_timer(self, host);
    }

    pub(super) fn start_viewport_animation_to<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        from_pan: CanvasPoint,
        from_zoom: f32,
        to_pan: CanvasPoint,
        to_zoom: f32,
        duration: std::time::Duration,
        interpolate: ViewportAnimationInterpolate,
        ease: Option<ViewportAnimationEase>,
    ) -> bool {
        super::viewport_timer_animation::start_viewport_animation_to(
            self,
            host,
            window,
            from_pan,
            from_zoom,
            to_pan,
            to_zoom,
            duration,
            interpolate,
            ease,
        )
    }

    pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
        super::viewport_timer_auto_pan::auto_pan_delta(snapshot, pos, bounds)
    }

    pub(super) fn stop_auto_pan_timer<H: UiHost>(&mut self, host: &mut H) {
        super::viewport_timer_auto_pan::stop_auto_pan_timer(self, host);
    }

    pub(super) fn stop_pan_inertia_timer<H: UiHost>(&mut self, host: &mut H) {
        super::viewport_timer_inertia::stop_pan_inertia_timer(self, host);
    }

    pub(super) fn bump_viewport_move_debounce<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
    ) {
        super::viewport_timer_animation::bump_viewport_move_debounce(
            self, host, window, snapshot, kind,
        );
    }

    pub(super) fn pan_inertia_should_tick(&self) -> bool {
        super::viewport_timer_inertia::pan_inertia_should_tick(self)
    }

    pub(super) fn maybe_start_pan_inertia_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        super::viewport_timer_inertia::maybe_start_pan_inertia_timer(self, host, window, snapshot)
    }

    pub(super) fn auto_pan_should_tick(&self, snapshot: &ViewSnapshot, bounds: Rect) -> bool {
        super::viewport_timer_auto_pan::auto_pan_should_tick(self, snapshot, bounds)
    }

    pub(super) fn sync_auto_pan_timer<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        bounds: Rect,
    ) {
        super::viewport_timer_auto_pan::sync_auto_pan_timer(self, host, window, snapshot, bounds);
    }
}
