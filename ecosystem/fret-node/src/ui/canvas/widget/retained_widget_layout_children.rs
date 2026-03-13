use super::*;

pub(super) fn layout_children<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    let internals_snapshot = canvas
        .internals
        .as_ref()
        .map(|store| store.snapshot())
        .unwrap_or_default();
    let anchors = canvas.diagnostics_anchor_ports.as_ref();
    let zero = Rect::new(cx.bounds.origin, Size::new(Px(0.0), Px(0.0)));
    for (index, &child) in cx.children.iter().enumerate() {
        if let Some(anchors) = anchors
            && index >= anchors.child_offset
            && index < anchors.child_offset.saturating_add(anchors.ports.len())
        {
            let port_index = index.saturating_sub(anchors.child_offset);
            let port = anchors.ports.get(port_index).copied();
            let rect = port
                .as_ref()
                .and_then(|port| internals_snapshot.ports_window.get(port).copied())
                .unwrap_or(zero);
            cx.layout_in(child, rect);
        } else {
            cx.layout_in(child, cx.bounds);
        }
    }
}
