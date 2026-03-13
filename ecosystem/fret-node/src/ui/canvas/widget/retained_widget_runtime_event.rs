use super::*;

pub(super) fn handle_retained_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
) {
    super::retained_widget_runtime_shared::sync_runtime_theme(
        canvas,
        cx.theme().snapshot(),
        Some(cx.services),
    );
    let snapshot = canvas.sync_view_state(cx.app);
    canvas.interaction.last_bounds = Some(cx.bounds);

    let outcome = {
        let middleware_cx = super::retained_widget_runtime_shared::middleware_cx(
            &canvas.graph,
            &canvas.view_state,
            &canvas.style,
            Some(cx.bounds),
            &snapshot,
        );
        canvas.middleware.handle_event(cx, &middleware_cx, event)
    };
    if outcome == NodeGraphCanvasEventOutcome::Handled {
        super::retained_widget_runtime_shared::finish_middleware_handled(cx);
        return;
    }

    canvas.handle_event(cx, event, &snapshot);
}
