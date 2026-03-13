use fret_core::Point;
use fret_ui::UiHost;

use crate::core::EdgeId;
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::ViewSnapshot;

use super::super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(in super::super) fn hit_hover_edge_anchor<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
    edge_id: EdgeId,
) -> Option<(EdgeId, EdgeEndpoint)> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let index = index.clone();
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
            canvas
                .hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                .filter(|(id, ..)| *id == edge_id)
                .map(|(id, endpoint, _fixed)| (id, endpoint))
        })
        .ok()
        .flatten()
}

pub(in super::super) fn hit_hover_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<EdgeId> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let index = index.clone();
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
            canvas.hit_edge(graph, snapshot, &mut ctx, position)
        })
        .ok()
        .flatten()
}
