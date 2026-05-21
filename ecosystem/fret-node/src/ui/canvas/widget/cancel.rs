use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith, cancel_cx::CancelGestureCx};

fn cancel_active_gestures_inner<H: UiHost, M: NodeGraphCanvasMiddleware, Cx: CancelGestureCx<H>>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    consume: bool,
) {
    let snapshot = canvas.sync_view_state(cx.host());
    let mode = snapshot.interaction.connection_mode;
    let mut canceled = false;

    canceled |= super::cancel_gesture_state::cancel_gesture_state(canvas, mode);
    canceled |= super::cancel_viewport_state::cancel_viewport_state(canvas, cx.host(), &snapshot);
    canceled |= super::cancel_cleanup::cancel_cleanup_state(canvas, mode);
    super::cancel_cleanup::clear_hover_and_focus(canvas);

    if canceled {
        canvas.stop_auto_pan_timer(cx.host());
        super::cancel_cleanup::finish_cancel(cx, consume);
    }
}

pub(super) fn cancel_active_gestures<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: CancelGestureCx<H>,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) {
    cancel_active_gestures_inner(canvas, cx, false);
}

pub(super) fn handle_escape_cancel<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: CancelGestureCx<H>,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) {
    cancel_active_gestures_inner(canvas, cx, true);
}
