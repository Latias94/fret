use super::*;

pub(super) fn handle_retained_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    command: &CommandId,
) -> bool {
    sync_runtime_theme(canvas, cx.theme().snapshot(), Some(cx.services));
    let snapshot = canvas.sync_view_state(cx.app);
    if should_defer_command_to_text_input(cx, command) {
        return false;
    }

    let outcome = {
        let middleware_cx = NodeGraphCanvasMiddlewareCx {
            graph: &canvas.graph,
            view_state: &canvas.view_state,
            style: &canvas.style,
            bounds: canvas.interaction.last_bounds,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        };
        canvas
            .middleware
            .handle_command(cx, &middleware_cx, command)
    };
    if outcome == NodeGraphCanvasCommandOutcome::Handled {
        finish_middleware_handled(cx);
        return true;
    }

    canvas.handle_command(cx, &snapshot, command)
}

pub(super) fn handle_retained_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
) {
    sync_runtime_theme(canvas, cx.theme().snapshot(), Some(cx.services));
    let snapshot = canvas.sync_view_state(cx.app);
    canvas.interaction.last_bounds = Some(cx.bounds);

    let outcome = {
        let middleware_cx = NodeGraphCanvasMiddlewareCx {
            graph: &canvas.graph,
            view_state: &canvas.view_state,
            style: &canvas.style,
            bounds: Some(cx.bounds),
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        };
        canvas.middleware.handle_event(cx, &middleware_cx, event)
    };
    if outcome == NodeGraphCanvasEventOutcome::Handled {
        finish_middleware_handled(cx);
        return;
    }

    canvas.handle_event(cx, event, &snapshot);
}

pub(super) fn paint_retained_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) {
    sync_runtime_theme(canvas, cx.theme().snapshot(), Some(cx.services));
    canvas.paint_root(cx);
}

fn should_defer_command_to_text_input<H: UiHost>(
    cx: &CommandCx<'_, H>,
    command: &CommandId,
) -> bool {
    cx.input_ctx.focus_is_text_input
        && (command.as_str().starts_with("node_graph.")
            || matches!(
                command.as_str(),
                "edit.copy" | "edit.cut" | "edit.paste" | "edit.select_all"
            ))
}

fn sync_runtime_theme<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    theme: fret_ui::ThemeSnapshot,
    services: Option<&mut dyn fret_core::UiServices>,
) {
    match services {
        Some(services) => {
            canvas.sync_style_from_color_mode(theme, Some(&mut *services));
            canvas.sync_skin(Some(&mut *services));
            canvas.sync_paint_overrides(Some(services));
        }
        None => {
            canvas.sync_style_from_color_mode(theme, None);
            canvas.sync_skin(None);
            canvas.sync_paint_overrides(None);
        }
    }
}

fn finish_middleware_handled<H: UiHost>(cx: &mut dyn WidgetCxLike<H>) {
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}

trait WidgetCxLike<H: UiHost> {
    fn stop_propagation(&mut self);
    fn request_redraw(&mut self);
    fn invalidate_self(&mut self, invalidation: Invalidation);
}

impl<H: UiHost> WidgetCxLike<H> for CommandCx<'_, H> {
    fn stop_propagation(&mut self) {
        CommandCx::stop_propagation(self);
    }

    fn request_redraw(&mut self) {
        CommandCx::request_redraw(self);
    }

    fn invalidate_self(&mut self, invalidation: Invalidation) {
        CommandCx::invalidate_self(self, invalidation);
    }
}

impl<H: UiHost> WidgetCxLike<H> for EventCx<'_, H> {
    fn stop_propagation(&mut self) {
        EventCx::stop_propagation(self);
    }

    fn request_redraw(&mut self) {
        EventCx::request_redraw(self);
    }

    fn invalidate_self(&mut self, invalidation: Invalidation) {
        EventCx::invalidate_self(self, invalidation);
    }
}
