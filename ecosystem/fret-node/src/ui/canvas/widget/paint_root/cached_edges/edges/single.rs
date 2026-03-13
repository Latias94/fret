use super::*;

pub(super) fn build_single_rect_edges_cache<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    edges_key: u64,
    edges_cache_rect: Rect,
    zoom: f32,
    view_interacting: bool,
    replay_delta: Point,
) {
    if canvas.try_replay_cached_edges(cx, edges_key, replay_delta) {
        canvas.edges_build_states.remove(&edges_key);
        return;
    }

    let mut state = canvas
        .edges_build_states
        .remove(&edges_key)
        .unwrap_or_else(|| {
            canvas.init_edges_build_state(
                &*cx.app,
                snapshot,
                geom,
                index,
                edges_cache_rect,
                edges_cache_rect,
                zoom,
            )
        });

    let wire_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let marker_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let mut wire_budget = WorkBudget::new(wire_budget_limit);
    let mut marker_budget = WorkBudget::new(marker_budget_limit);

    let mut tmp = fret_core::Scene::default();
    if canvas.paint_edges_build_state_step(
        &mut tmp,
        &*cx.app,
        cx.services,
        zoom,
        cx.scale_factor,
        &mut state,
        &mut wire_budget,
        &mut marker_budget,
    ) {
        super::super::super::redraw_request::request_paint_redraw(cx);
    }

    if state.edges.is_empty() {
        canvas.edges_scene_cache.store_ops(edges_key, Vec::new());
    } else if state.ops.len() > 2 {
        canvas.replay_cached_edge_build_state(cx, &state, replay_delta);
        if state.next_edge >= state.edges.len() {
            canvas.store_finished_edge_build_state(edges_key, state);
        } else {
            canvas.edges_build_states.insert(edges_key, state);
        }
    } else {
        canvas.paint_cache.touch_paths_in_scene_ops(&state.ops);
        canvas.edges_build_states.insert(edges_key, state);
    }
}
