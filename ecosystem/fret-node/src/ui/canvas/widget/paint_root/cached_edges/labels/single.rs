use super::*;

pub(super) fn build_single_rect_edge_labels_cache<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    labels_key: u64,
    edges_cache_rect: Rect,
    zoom: f32,
    view_interacting: bool,
) {
    if canvas.edge_labels_scene_cache.contains_key(labels_key) {
        canvas.edge_labels_build_state = None;
        return;
    }

    let mut state = canvas
        .edge_labels_build_state
        .take()
        .filter(|state| state.key == labels_key)
        .unwrap_or_else(|| {
            canvas.init_edge_labels_build_state(
                &*cx.app,
                snapshot,
                geom,
                index,
                labels_key,
                edges_cache_rect,
                edges_cache_rect,
                zoom,
            )
        });

    let budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let mut budget = WorkBudget::new(budget_limit);
    let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

    let mut tmp = fret_core::Scene::default();
    if canvas.paint_edge_labels_build_state_step(
        &mut tmp,
        &*cx.app,
        cx.services,
        cx.scale_factor,
        zoom,
        bezier_steps,
        &mut state,
        &mut budget,
    ) {
        crate::ui::canvas::widget::redraw_request::request_paint_redraw(cx);
    }

    if state.next_edge >= state.edges.len() {
        canvas
            .edge_labels_scene_cache
            .store_ops(labels_key, state.ops.clone());
        canvas.edge_labels_build_state = None;
    } else {
        canvas.edge_labels_build_state = Some(state);
    }
}
