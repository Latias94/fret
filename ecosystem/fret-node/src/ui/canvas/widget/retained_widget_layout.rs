use super::*;

pub(super) fn layout_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    let theme = cx.theme().snapshot();
    canvas.sync_style_from_color_mode(theme, Some(cx.services));
    canvas.sync_skin(Some(cx.services));
    super::retained_widget_layout_observe::observe_layout_models(canvas, cx);
    canvas.interaction.last_bounds = Some(cx.bounds);
    let snapshot = canvas.sync_view_state(cx.app);

    canvas.update_auto_measured_node_sizes(cx);
    super::retained_widget_layout_publish::publish_diagnostics_derived_outputs(
        canvas, cx, &snapshot,
    );
    super::retained_widget_layout_children::layout_children(canvas, cx);
    super::retained_widget_layout_drain::drain_post_layout_queues(canvas, cx)
}
