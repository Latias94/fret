use super::*;

pub(super) fn paint_edges_build_state_step<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    services: &mut dyn fret_core::UiServices,
    zoom: f32,
    scale_factor: f32,
    state: &mut EdgesBuildState,
    wire_budget: &mut WorkBudget,
    marker_budget: &mut WorkBudget,
) -> bool {
    let mut tmp = super::temp_scene::paint_root_cached_edge_build_state_temp_scene();
    let (next_edge, skipped) = canvas.paint_edges_cached_budgeted(
        &mut tmp,
        host,
        services,
        &state.edges,
        zoom,
        scale_factor,
        state.next_edge,
        wire_budget,
        marker_budget,
    );
    super::ops::finish_build_state_step(
        &mut state.ops,
        state.edges.len(),
        &mut state.next_edge,
        &tmp,
        next_edge,
        skipped,
    )
}

pub(super) fn paint_edge_labels_build_state_step<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &H,
    services: &mut dyn fret_core::UiServices,
    scale_factor: f32,
    zoom: f32,
    bezier_steps: usize,
    state: &mut EdgeLabelsBuildState,
    budget: &mut WorkBudget,
) -> bool {
    let mut tmp = super::temp_scene::paint_root_cached_edge_build_state_temp_scene();
    let (next_edge, skipped) = canvas.paint_edge_labels_static_budgeted_cached(
        &mut tmp,
        host,
        services,
        scale_factor,
        &state.edges,
        bezier_steps,
        zoom,
        state.next_edge,
        budget,
    );
    super::ops::finish_build_state_step(
        &mut state.ops,
        state.edges.len(),
        &mut state.next_edge,
        &tmp,
        next_edge,
        skipped,
    )
}
