use super::*;

pub(super) fn layout_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    let theme = cx.theme().snapshot();
    canvas.sync_style_from_color_mode(theme, Some(cx.services));
    canvas.sync_skin(Some(cx.services));
    observe_layout_models(canvas, cx);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    canvas.update_auto_measured_node_sizes(cx);
    publish_diagnostics_derived_outputs(canvas, cx, &snapshot);
    layout_children(canvas, cx);
    drain_post_layout_queues(canvas, cx)
}

fn publish_diagnostics_derived_outputs<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
    snapshot: &ViewSnapshot,
) {
    if canvas.diagnostics_anchor_ports.is_some() {
        let (geometry, _index) = canvas.canvas_derived(&*cx.app, snapshot);
        canvas.publish_derived_outputs(&*cx.app, snapshot, cx.bounds, &geometry);
    }
}

fn drain_post_layout_queues<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    canvas.drain_edit_queue(cx.app, cx.window);
    let did_view_queue = canvas.drain_view_queue(cx.app, cx.window);
    let did_fit_on_mount =
        canvas.maybe_fit_view_on_mount(cx.app, cx.window, cx.bounds, did_view_queue);
    if did_view_queue || did_fit_on_mount {
        cx.request_redraw();
    }
    cx.available
}

fn observe_layout_models<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    cx.observe_model(&canvas.graph, Invalidation::Layout);
    cx.observe_model(&canvas.view_state, Invalidation::Layout);
    if let Some(queue) = canvas.edit_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
    if let Some(queue) = canvas.view_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
}

fn layout_children<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
