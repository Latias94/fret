use fret_core::Point;
use fret_ui::UiHost;

use super::super::{HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) enum StickyWireNonPortTarget {
    Node,
    Edge(crate::core::EdgeId),
    Canvas,
}

pub(super) fn inspect_non_port_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &crate::ui::canvas::CanvasGeometry,
    index: &crate::ui::canvas::CanvasSpatialDerived,
    position: Point,
    zoom: f32,
) -> StickyWireNonPortTarget {
    canvas
        .graph
        .read_ref(host, |graph| {
            let on_node = geom.order.iter().rev().any(|id| {
                geom.nodes
                    .get(id)
                    .is_some_and(|node| node.rect.contains(position))
            });
            if on_node {
                return StickyWireNonPortTarget::Node;
            }
            let mut scratch = HitTestScratch::default();
            let mut hit_test = HitTestCtx::new(geom, index, zoom, &mut scratch);
            canvas
                .hit_edge(graph, snapshot, &mut hit_test, position)
                .map_or(
                    StickyWireNonPortTarget::Canvas,
                    StickyWireNonPortTarget::Edge,
                )
        })
        .ok()
        .unwrap_or(StickyWireNonPortTarget::Canvas)
}
