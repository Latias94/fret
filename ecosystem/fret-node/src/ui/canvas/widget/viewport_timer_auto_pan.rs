mod delta;
mod policy;
mod timer;

use super::*;

pub(super) fn auto_pan_delta(snapshot: &ViewSnapshot, pos: Point, bounds: Rect) -> CanvasPoint {
    delta::auto_pan_delta(snapshot, pos, bounds)
}

pub(super) fn stop_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    timer::stop_auto_pan_timer(canvas, host);
}

pub(super) fn auto_pan_should_tick<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) -> bool {
    policy::auto_pan_should_tick(canvas, snapshot, bounds)
}

pub(super) fn sync_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
    bounds: Rect,
) {
    if policy::auto_pan_should_tick(canvas, snapshot, bounds) {
        timer::ensure_auto_pan_timer_running(canvas, host, window);
    } else {
        timer::stop_auto_pan_timer(canvas, host);
    }
}
