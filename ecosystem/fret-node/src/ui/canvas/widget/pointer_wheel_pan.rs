mod apply;
mod gate;
mod resolve;

use super::*;

pub(super) fn handle_scroll_pan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    delta: Point,
    modifiers: fret_core::Modifiers,
) -> bool {
    if !gate::scroll_pan_enabled(canvas, snapshot) {
        return false;
    }

    let resolved = resolve::resolve_scroll_pan(snapshot, cx.input_ctx.platform, delta, modifiers);
    apply::apply_scroll_pan(canvas, cx, snapshot, resolved);
    true
}
