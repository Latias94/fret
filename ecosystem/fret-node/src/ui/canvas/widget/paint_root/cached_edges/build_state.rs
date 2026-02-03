use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

fn extend_clip_stack_ops(ops: &mut Vec<SceneOp>, tmp: &[SceneOp]) {
    if tmp.is_empty() {
        return;
    }

    match ops.pop() {
        Some(SceneOp::PopClip) => {
            ops.extend_from_slice(tmp);
            ops.push(SceneOp::PopClip);
        }
        Some(other) => {
            ops.push(other);
            ops.extend_from_slice(tmp);
        }
        None => {
            ops.extend_from_slice(tmp);
        }
    }

    if !matches!(ops.last(), Some(SceneOp::PopClip)) {
        ops.push(SceneOp::PopClip);
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn init_edges_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgesBuildState {
        let render_edges: RenderData = self.collect_render_data(
            host,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
            Some(cull_rect),
            zoom,
            None,
            false,
            false,
            true,
        );
        EdgesBuildState {
            ops: vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip],
            edges: render_edges.edges,
            next_edge: 0,
        }
    }

    pub(super) fn init_edge_labels_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
        key: u64,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgeLabelsBuildState {
        let render_edges: RenderData = self.collect_render_data(
            host,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
            Some(cull_rect),
            zoom,
            None,
            false,
            false,
            true,
        );
        EdgeLabelsBuildState {
            key,
            ops: vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip],
            edges: render_edges.edges,
            next_edge: 0,
        }
    }

    pub(super) fn paint_edges_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        state: &mut EdgesBuildState,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
    ) -> bool {
        let (next_edge, skipped) = self.paint_edges_cached_budgeted(
            tmp,
            host,
            services,
            &state.edges,
            zoom,
            scale_factor,
            state.next_edge,
            wire_budget,
            marker_budget,
        );
        state.next_edge = next_edge;

        extend_clip_stack_ops(&mut state.ops, tmp.ops());
        skipped || state.next_edge < state.edges.len()
    }

    pub(super) fn paint_edge_labels_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        zoom: f32,
        bezier_steps: usize,
        state: &mut EdgeLabelsBuildState,
        budget: &mut WorkBudget,
    ) -> bool {
        let (next_edge, skipped) = self.paint_edge_labels_static_budgeted_cached(
            tmp,
            host,
            services,
            scale_factor,
            &state.edges,
            bezier_steps,
            zoom,
            state.next_edge,
            budget,
        );
        state.next_edge = next_edge;

        extend_clip_stack_ops(&mut state.ops, tmp.ops());
        skipped || state.next_edge < state.edges.len()
    }
}
