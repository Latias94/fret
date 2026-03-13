use super::*;

pub(super) fn handle_retained_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    command: &CommandId,
) -> bool {
    super::retained_widget_runtime_command::handle_retained_command(canvas, cx, command)
}

pub(super) fn handle_retained_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
) {
    super::retained_widget_runtime_event::handle_retained_event(canvas, cx, event);
}

pub(super) fn paint_retained_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) {
    super::retained_widget_runtime_paint::paint_retained_widget(canvas, cx);
}
