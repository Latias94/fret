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

fn initial_clip_ops(clip_rect: Rect) -> Vec<SceneOp> {
    vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip]
}

fn finish_build_state_step(
    ops: &mut Vec<SceneOp>,
    edge_count: usize,
    next_edge_slot: &mut usize,
    tmp: &fret_core::Scene,
    next_edge: usize,
    skipped: bool,
) -> bool {
    *next_edge_slot = next_edge;
    extend_clip_stack_ops(ops, tmp.ops());
    skipped || *next_edge_slot < edge_count
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn collect_cached_edge_renders<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        cull_rect: Rect,
        zoom: f32,
    ) -> Vec<paint_render_data::EdgeRender> {
        self.collect_render_data(
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
        )
        .edges
    }

    pub(super) fn init_edges_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgesBuildState {
        let (ops, edges) = self
            .init_cached_edge_build_parts(host, snapshot, geom, index, clip_rect, cull_rect, zoom);
        EdgesBuildState {
            ops,
            edges,
            next_edge: 0,
        }
    }

    pub(super) fn init_edge_labels_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        key: u64,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgeLabelsBuildState {
        let (ops, edges) = self
            .init_cached_edge_build_parts(host, snapshot, geom, index, clip_rect, cull_rect, zoom);
        EdgeLabelsBuildState {
            key,
            ops,
            edges,
            next_edge: 0,
        }
    }

    fn init_cached_edge_build_parts<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> (Vec<SceneOp>, Vec<paint_render_data::EdgeRender>) {
        (
            initial_clip_ops(clip_rect),
            self.collect_cached_edge_renders(host, snapshot, geom, index, cull_rect, zoom),
        )
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
        finish_build_state_step(
            &mut state.ops,
            state.edges.len(),
            &mut state.next_edge,
            tmp,
            next_edge,
            skipped,
        )
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
        finish_build_state_step(
            &mut state.ops,
            state.edges.len(),
            &mut state.next_edge,
            tmp,
            next_edge,
            skipped,
        )
    }
}
