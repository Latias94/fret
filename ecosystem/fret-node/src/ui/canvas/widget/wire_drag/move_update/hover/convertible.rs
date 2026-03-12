use super::super::prelude::*;

pub(super) fn compute_hover_convertible<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
    hover_port: Option<PortId>,
    hover_valid: bool,
) -> bool {
    if hover_valid {
        return false;
    }

    let Some(target) = hover_port else {
        return false;
    };

    let WireDragKind::New { from, bundle } = kind else {
        return false;
    };
    if bundle.len() > 1 {
        return false;
    }

    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            if !NodeGraphCanvasWith::<M>::port_is_connectable_end(
                graph,
                &snapshot.interaction,
                target,
            ) {
                return false;
            }
            conversion::is_convertible(presenter, graph, *from, target)
        })
        .ok()
        .unwrap_or(false)
}
