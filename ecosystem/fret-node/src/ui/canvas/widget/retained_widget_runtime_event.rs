use super::*;

impl<H: UiHost, M: NodeGraphCanvasMiddleware> event_runtime_adapter::CanvasEventRuntimeCx<H, M>
    for EventCx<'_, H>
{
    fn event_runtime_theme_snapshot(&self) -> fret_ui::ThemeSnapshot {
        self.theme().snapshot()
    }

    fn event_runtime_services(&mut self) -> Option<&mut dyn fret_core::UiServices> {
        Some(self.services)
    }

    fn event_runtime_host(&mut self) -> &mut H {
        self.app
    }

    fn event_runtime_bounds(&self) -> Rect {
        self.bounds
    }
}

pub(super) fn handle_retained_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
) {
    event_runtime_adapter::dispatch_canvas_event(canvas, cx, event);
}
