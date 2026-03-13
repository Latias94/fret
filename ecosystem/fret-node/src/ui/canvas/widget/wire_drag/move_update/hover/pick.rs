use super::super::prelude::*;

pub(super) fn pick_hover_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    zoom: f32,
    from_port: Option<PortId>,
    require_from_connectable_start: bool,
    pos: Point,
) -> Option<PortId> {
    let Some(from_port) = from_port else {
        return None;
    };

    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom, index, zoom, &mut scratch);
            canvas.pick_wire_hover_port(
                graph,
                snapshot,
                &mut ctx,
                from_port,
                require_from_connectable_start,
                pos,
            )
        })
        .ok()
        .flatten()
}

pub(super) fn pick_hover_edge_if_no_hover_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    zoom: f32,
    pos: Point,
    hover_port: Option<PortId>,
) -> Option<EdgeId> {
    if hover_port.is_some() {
        return None;
    }

    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom, index, zoom, &mut scratch);
            canvas.hit_edge(graph, snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten()
}
