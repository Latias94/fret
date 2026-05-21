use crate::ui::canvas::widget::*;

use super::resolve::ResolvedScrollPan;

pub(super) fn apply_scroll_pan<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    resolved: ResolvedScrollPan,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    let window = cx.window();
    canvas.bump_viewport_move_debounce(cx.host(), window, snapshot, ViewportMoveKind::PanScroll);
    canvas.update_view_state(cx.host(), |state| {
        state.pan.x += resolved.dx * resolved.speed;
        state.pan.y += resolved.dy * resolved.speed;
    });
    super::super::paint_invalidation::invalidate_paint(cx);
}
