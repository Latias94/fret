use super::*;

pub(super) fn prepaint_cull_window<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PrepaintCx<'_, H>,
) {
    let snapshot = canvas.sync_view_state(cx.app);
    if !super::retained_widget_cull_window_key::should_track_cull_window(canvas, &snapshot) {
        return;
    }

    let Some(next_key) =
        super::retained_widget_cull_window_key::build_cull_window_key(cx.bounds, &snapshot)
    else {
        return;
    };

    super::retained_widget_cull_window_shift::apply_cull_window_key(canvas, cx, next_key);
}
