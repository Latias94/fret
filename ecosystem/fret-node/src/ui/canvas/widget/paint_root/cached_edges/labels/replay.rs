use super::*;

fn replay_edge_label_ops<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    paint_cache: &mut CanvasPaintCache,
    ops: &[SceneOp],
    replay_delta: Point,
) {
    cx.scene.replay_ops_translated(ops, replay_delta);
    paint_cache.touch_text_blobs_in_scene_ops(ops);
}

pub(super) fn store_finished_edge_label_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    key: u64,
    state: EdgeLabelsBuildState,
) {
    if state.ops.len() == 2 {
        canvas.edge_labels_scene_cache.store_ops(key, Vec::new());
    } else {
        canvas.edge_labels_scene_cache.store_ops(key, state.ops);
    }
}

pub(super) fn try_replay_cached_edge_labels<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    key: u64,
    replay_delta: Point,
) -> bool {
    canvas
        .edge_labels_scene_cache
        .try_replay_with(key, cx.scene, replay_delta, |ops| {
            canvas.paint_cache.touch_text_blobs_in_scene_ops(ops);
        })
}

pub(super) fn replay_single_rect_edge_labels<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    labels_key: u64,
    replay_delta: Point,
) {
    if try_replay_cached_edge_labels(canvas, cx, labels_key, replay_delta) {
        return;
    }
    if let Some(state) = canvas
        .edge_labels_build_state
        .as_ref()
        .filter(|state| state.key == labels_key)
    {
        replay_edge_label_ops(cx, &mut canvas.paint_cache, &state.ops, replay_delta);
    }
}

pub(super) fn replay_partial_edge_label_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    ops: &[SceneOp],
    replay_delta: Point,
) {
    replay_edge_label_ops(cx, &mut canvas.paint_cache, ops, replay_delta);
}
