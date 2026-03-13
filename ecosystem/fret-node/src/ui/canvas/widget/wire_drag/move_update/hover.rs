mod convertible;
mod pick;
mod source;
mod validity;

use super::prelude::*;

pub(super) fn from_port_and_require_from_connectable_start(
    kind: &WireDragKind,
) -> (Option<PortId>, bool) {
    source::from_port_and_require_from_connectable_start(kind)
}

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
    pick::pick_hover_port(
        canvas,
        host,
        snapshot,
        geom,
        index,
        zoom,
        from_port,
        require_from_connectable_start,
        pos,
    )
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
    pick::pick_hover_edge_if_no_hover_port(
        canvas, host, snapshot, geom, index, zoom, pos, hover_port,
    )
}

pub(super) fn compute_hover_validity_and_diag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
    hover_port: Option<PortId>,
) -> (bool, Option<(DiagnosticSeverity, Arc<str>)>) {
    validity::compute_hover_validity_and_diag(canvas, host, snapshot, kind, hover_port)
}

pub(super) fn compute_hover_convertible<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
    hover_port: Option<PortId>,
    hover_valid: bool,
) -> bool {
    convertible::compute_hover_convertible(canvas, host, snapshot, kind, hover_port, hover_valid)
}
