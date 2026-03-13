use crate::ui::canvas::widget::*;

use super::resolve::ResolvedScrollPan;

pub(super) fn apply_scroll_pan<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    resolved: ResolvedScrollPan,
) {
    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::PanScroll);
    canvas.update_view_state(cx.app, |state| {
        state.pan.x += resolved.dx * resolved.speed;
        state.pan.y += resolved.dy * resolved.speed;
    });
    super::super::paint_invalidation::invalidate_paint(cx);
}
