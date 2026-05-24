mod apply;
mod gate;
mod resolve;

use super::*;

pub(super) fn handle_scroll_pan<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    platform: fret_runtime::Platform,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    if !gate::scroll_pan_enabled(canvas, snapshot) {
        return false;
    }

    let resolved = resolve::resolve_scroll_pan(snapshot, platform, delta, modifiers);
    apply::apply_scroll_pan(canvas, cx, snapshot, resolved);
    true
}
