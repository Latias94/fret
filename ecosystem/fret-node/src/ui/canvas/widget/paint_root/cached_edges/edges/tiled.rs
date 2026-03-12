use super::super::geometry::cache_tile_rect;
use super::super::keys;
use super::*;

pub(super) fn paint_tiled_edges_cache<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    tiles: &[TileCoord],
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
    zoom: f32,
    view_interacting: bool,
    replay_delta: Point,
) {
    let edges_base_key =
        keys::edges_tiles_base_key(base_key, style_key, edges_cache_tile_size_canvas);

    let wire_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let marker_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let tile_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
    let mut wire_budget = WorkBudget::new(wire_budget_limit);
    let mut marker_budget = WorkBudget::new(marker_budget_limit);
    let mut tile_budget = WorkBudget::new(tile_budget_limit);

    let mut skipped = false;

    for tile in tiles.iter().copied() {
        let tile_key = tile_cache_key(edges_base_key, tile);
        canvas.edges_tile_keys_scratch.push(tile_key);

        if canvas.try_replay_cached_edges(cx, tile_key, replay_delta) {
            canvas.edges_build_states.remove(&tile_key);
            continue;
        }

        if !tile_budget.try_consume(1) {
            skipped = true;
            continue;
        }

        let tile_rect = cache_tile_rect(tile, edges_cache_tile_size_canvas);
        let tile_cull_rect = canvas.cache_tile_cull_rect(tile_rect, zoom);

        let mut state = canvas
            .edges_build_states
            .remove(&tile_key)
            .unwrap_or_else(|| {
                canvas.init_edges_build_state(
                    &*cx.app,
                    snapshot,
                    geom,
                    index,
                    tile_rect,
                    tile_cull_rect,
                    zoom,
                )
            });

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
            skipped = true;
        }

        if state.edges.is_empty() {
            canvas.edges_scene_cache.store_ops(tile_key, Vec::new());
            continue;
        }

        if state.ops.len() > 2 {
            canvas.replay_cached_edge_build_state(cx, &state, replay_delta);
        }

        if state.next_edge >= state.edges.len() {
            canvas.store_finished_edge_build_state(tile_key, state);
        } else {
            canvas.edges_build_states.insert(tile_key, state);
        }
    }

    canvas
        .edges_build_states
        .retain(|key, _| canvas.edges_tile_keys_scratch.contains(key));

    super::super::super::redraw_request::request_paint_redraw_if(cx, skipped);
}
