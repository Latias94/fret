use super::*;

pub(super) fn replay_cached_edge_build_state<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    state: &EdgesBuildState,
    replay_delta: Point,
) {
    cx.scene.replay_ops_translated(&state.ops, replay_delta);
    canvas.paint_cache.touch_paths_in_scene_ops(&state.ops);
}

pub(super) fn store_finished_edge_build_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    key: u64,
    state: EdgesBuildState,
) {
    canvas.edges_scene_cache.store_ops(key, state.ops);
}

pub(super) fn try_replay_cached_edges<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    key: u64,
    replay_delta: Point,
) -> bool {
    canvas
        .edges_scene_cache
        .try_replay_with(key, cx.scene, replay_delta, |ops| {
            canvas.paint_cache.touch_paths_in_scene_ops(ops);
        })
}
