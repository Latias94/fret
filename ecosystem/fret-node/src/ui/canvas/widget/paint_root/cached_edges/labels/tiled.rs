use super::super::geometry::cache_tile_rect;
use super::super::keys;
use super::*;

pub(super) fn paint_tiled_edge_labels_cache<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
    canvas.edge_labels_tile_keys_scratch.clear();

    let labels_base_key =
        keys::edge_labels_tiles_base_key(base_key, style_key, edges_cache_tile_size_canvas);
    let tile_budget_limit = NodeGraphCanvasWith::<M>::EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME
        .select(view_interacting);
    let label_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let mut tile_budget = WorkBudget::new(tile_budget_limit);
    let mut label_budget = WorkBudget::new(label_budget_limit);

    let mut skipped_labels = false;
    let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

    for tile in tiles.iter().copied() {
        let tile_key = tile_cache_key(labels_base_key, tile);
        canvas.edge_labels_tile_keys_scratch.push(tile_key);

        if canvas.try_replay_cached_edge_labels(cx, tile_key, replay_delta) {
            canvas.edge_labels_build_states.remove(&tile_key);
            continue;
        }

        if !tile_budget.try_consume(1) {
            skipped_labels = true;
            continue;
        }

        let tile_rect = cache_tile_rect(tile, edges_cache_tile_size_canvas);
        let tile_cull_rect = canvas.cache_tile_cull_rect(tile_rect, zoom);

        let mut state = canvas
            .edge_labels_build_states
            .remove(&tile_key)
            .unwrap_or_else(|| {
                canvas.init_edge_labels_build_state(
                    &*cx.app,
                    snapshot,
                    geom,
                    index,
                    tile_key,
                    tile_rect,
                    tile_cull_rect,
                    zoom,
                )
            });

        if state.edges.is_empty() {
            canvas
                .edge_labels_scene_cache
                .store_ops(tile_key, Vec::new());
            continue;
        }

        let mut tmp = fret_core::Scene::default();
        if canvas.paint_edge_labels_build_state_step(
            &mut tmp,
            &*cx.app,
            cx.services,
            cx.scale_factor,
            zoom,
            bezier_steps,
            &mut state,
            &mut label_budget,
        ) {
            skipped_labels = true;
        }

        if state.ops.len() > 2 {
            super::replay::replay_partial_edge_label_ops(canvas, cx, &state.ops, replay_delta);
        }

        if state.next_edge >= state.edges.len() {
            canvas.store_finished_edge_label_state(tile_key, state);
        } else {
            canvas.edge_labels_build_states.insert(tile_key, state);
        }
    }

    canvas
        .edge_labels_build_states
        .retain(|key, _| canvas.edge_labels_tile_keys_scratch.contains(key));

    crate::ui::canvas::widget::redraw_request::request_paint_redraw_if(cx, skipped_labels);
}
