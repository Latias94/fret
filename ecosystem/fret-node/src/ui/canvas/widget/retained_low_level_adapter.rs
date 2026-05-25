//! Retained-context bindings for the low-level canvas adapter contract.

use super::*;

impl<H: UiHost> low_level_adapter::CanvasRedrawCx<H> for EventCx<'_, H> {
    fn request_redraw(&mut self) {
        EventCx::request_redraw(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasPaintInvalidationCx<H> for EventCx<'_, H> {
    fn invalidate_paint(&mut self) {
        EventCx::invalidate_self(self, Invalidation::Paint);
    }
}

impl<H: UiHost> low_level_adapter::CanvasHandledCx<H> for EventCx<'_, H> {
    fn stop_propagation(&mut self) {
        EventCx::stop_propagation(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasPointerCaptureReleaseCx<H> for EventCx<'_, H> {
    fn release_pointer_capture(&mut self) {
        EventCx::release_pointer_capture(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasRedrawCx<H> for CommandCx<'_, H> {
    fn request_redraw(&mut self) {
        CommandCx::request_redraw(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasPaintInvalidationCx<H> for CommandCx<'_, H> {
    fn invalidate_paint(&mut self) {
        CommandCx::invalidate_self(self, Invalidation::Paint);
    }
}

impl<H: UiHost> low_level_adapter::CanvasHandledCx<H> for CommandCx<'_, H> {
    fn stop_propagation(&mut self) {
        CommandCx::stop_propagation(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasRedrawCx<H> for LayoutCx<'_, H> {
    fn request_redraw(&mut self) {
        LayoutCx::request_redraw(self);
    }
}

impl<H: UiHost> low_level_adapter::CanvasRedrawCx<H> for PaintCx<'_, H> {
    fn request_redraw(&mut self) {
        PaintCx::request_redraw(self);
    }
}
