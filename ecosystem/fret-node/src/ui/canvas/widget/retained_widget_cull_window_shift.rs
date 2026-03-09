use super::*;

pub(super) fn apply_cull_window_key<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PrepaintCx<'_, H>,
    next_key: u64,
) {
    match canvas.last_cull_window_key {
        None => {
            canvas.last_cull_window_key = Some(next_key);
        }
        Some(prev_key) if prev_key != next_key => {
            cx.debug_record_node_graph_cull_window_shift(next_key);
            canvas.last_cull_window_key = Some(next_key);
        }
        _ => {}
    }
}
