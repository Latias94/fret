use super::*;

impl<H: UiHost> widget_tail::WidgetRedrawCx<H> for EventCx<'_, H> {
    fn request_redraw(&mut self) {
        EventCx::request_redraw(self);
    }
}

impl<H: UiHost> widget_tail::WidgetPaintInvalidationCx<H> for EventCx<'_, H> {
    fn invalidate_paint(&mut self) {
        EventCx::invalidate_self(self, Invalidation::Paint);
    }
}

impl<H: UiHost> widget_tail::WidgetHandledCx<H> for EventCx<'_, H> {
    fn stop_propagation(&mut self) {
        EventCx::stop_propagation(self);
    }
}

impl<H: UiHost> widget_tail::PointerCaptureReleaseCx<H> for EventCx<'_, H> {
    fn release_pointer_capture(&mut self) {
        EventCx::release_pointer_capture(self);
    }
}

impl<H: UiHost> widget_tail::WidgetRedrawCx<H> for CommandCx<'_, H> {
    fn request_redraw(&mut self) {
        CommandCx::request_redraw(self);
    }
}

impl<H: UiHost> widget_tail::WidgetPaintInvalidationCx<H> for CommandCx<'_, H> {
    fn invalidate_paint(&mut self) {
        CommandCx::invalidate_self(self, Invalidation::Paint);
    }
}

impl<H: UiHost> widget_tail::WidgetHandledCx<H> for CommandCx<'_, H> {
    fn stop_propagation(&mut self) {
        CommandCx::stop_propagation(self);
    }
}

impl<H: UiHost> widget_tail::WidgetRedrawCx<H> for LayoutCx<'_, H> {
    fn request_redraw(&mut self) {
        LayoutCx::request_redraw(self);
    }
}

impl<H: UiHost> widget_tail::WidgetRedrawCx<H> for PaintCx<'_, H> {
    fn request_redraw(&mut self) {
        PaintCx::request_redraw(self);
    }
}
