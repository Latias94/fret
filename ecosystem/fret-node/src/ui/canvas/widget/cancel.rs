use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

fn cancel_active_gestures_inner<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    consume: bool,
) {
    let snapshot = canvas.sync_view_state(cx.app);
    let mode = snapshot.interaction.connection_mode;
    let mut canceled = false;

    canceled |= super::cancel_gesture_state::cancel_gesture_state(canvas, mode);
    canceled |= super::cancel_viewport_state::cancel_viewport_state(canvas, cx, &snapshot);
    canceled |= super::cancel_cleanup::cancel_cleanup_state(canvas, mode);
    super::cancel_cleanup::clear_hover_and_focus(canvas);

    if canceled {
        super::cancel_cleanup::finish_cancel(canvas, cx, consume);
    }
}

pub(super) fn cancel_active_gestures<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cancel_active_gestures_inner(canvas, cx, false);
}

pub(super) fn handle_escape_cancel<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    cancel_active_gestures_inner(canvas, cx, true);
}
