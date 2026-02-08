use super::prelude::*;

pub(super) fn maybe_extend_bundle_on_shift<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    modifiers: Modifiers,
    zoom: f32,
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    pos: Point,
    kind: &mut WireDragKind,
) {
    if !modifiers.shift {
        return;
    }

    let WireDragKind::New { from, bundle } = kind else {
        return;
    };

    let mut scratch = HitTestScratch::default();
    let mut ctx = HitTestCtx::new(geom, index, zoom, &mut scratch);
    let candidate = canvas.hit_port(&mut ctx, pos);

    let Some(candidate) = candidate else {
        return;
    };

    let should_add = canvas
        .graph
        .read_ref(host, |graph| {
            if !NodeGraphCanvasWith::<M>::port_is_connectable_start(
                graph,
                &snapshot.interaction,
                candidate,
            ) {
                return false;
            }
            NodeGraphCanvasWith::<M>::should_add_bundle_port(graph, *from, bundle, candidate)
        })
        .ok()
        .unwrap_or(false);

    if should_add {
        bundle.push(candidate);
    }
}
