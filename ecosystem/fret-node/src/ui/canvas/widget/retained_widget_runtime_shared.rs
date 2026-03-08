use super::*;

pub(super) fn middleware_cx<'a>(
    graph: &'a Model<Graph>,
    view_state: &'a Model<NodeGraphViewState>,
    style: &'a NodeGraphStyle,
    bounds: Option<Rect>,
    snapshot: &'a ViewSnapshot,
) -> NodeGraphCanvasMiddlewareCx<'a> {
    NodeGraphCanvasMiddlewareCx {
        graph,
        view_state,
        style,
        bounds,
        pan: snapshot.pan,
        zoom: snapshot.zoom,
    }
}

pub(super) fn sync_runtime_theme<M: NodeGraphCanvasMiddleware>(
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

pub(super) fn finish_middleware_handled<H: UiHost, C: WidgetCxLike<H>>(cx: &mut C) {
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}

pub(super) trait WidgetCxLike<H: UiHost> {
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
