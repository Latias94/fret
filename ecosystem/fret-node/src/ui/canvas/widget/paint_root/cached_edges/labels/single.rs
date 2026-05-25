use super::*;

pub(super) fn build_single_rect_edge_labels_cache<H, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    labels_key: u64,
    edges_cache_rect: Rect,
    zoom: f32,
    view_interacting: bool,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: super::super::label_build_state_adapter::PaintRootCachedEdgeLabelBuildStateCx<H>
        + crate::ui::canvas::widget::low_level_adapter::CanvasRedrawCx<H>,
{
    if canvas.edge_labels_scene_cache.contains_key(labels_key) {
        canvas.edge_labels_build_state = None;
        return;
    }

    let mut state = canvas
        .edge_labels_build_state
        .take()
        .filter(|state| state.key == labels_key)
        .unwrap_or_else(|| {
            let host = super::super::label_build_state_adapter::
                paint_root_cached_edge_label_build_state_host(&*cx);
            canvas.init_edge_labels_build_state(
                host,
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
    let needs_more = {
        let inputs = super::super::label_build_state_adapter::
            paint_root_cached_edge_label_build_state_step_inputs(cx);
        canvas.paint_edge_labels_build_state_step(
            &mut tmp,
            inputs.host,
            inputs.services,
            inputs.scale_factor,
            zoom,
            bezier_steps,
            &mut state,
            &mut budget,
        )
    };
    if needs_more {
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
