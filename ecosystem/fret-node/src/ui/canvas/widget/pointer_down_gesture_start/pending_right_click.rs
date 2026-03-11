use crate::ui::canvas::widget::*;

pub(super) fn handle_pending_right_click_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
) -> bool {
    if button != MouseButton::Right {
        return false;
    }

    cancel::cancel_active_gestures(canvas, cx);
    if !snapshot.interaction.pan_on_drag.right {
        return false;
    }

    canvas.interaction.pending_right_click = Some(crate::ui::canvas::state::PendingRightClick {
        start_pos: position,
    });
    cx.capture_pointer(cx.node);
    paint_invalidation::invalidate_paint(cx);
    true
}
