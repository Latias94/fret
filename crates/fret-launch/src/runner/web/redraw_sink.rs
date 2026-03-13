use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    /// Backend-local redraw sinks for queued events or resource/window sync.
    /// These intentionally do not write runner frame-drive diagnostics.
    pub(super) fn request_sink_redraw(&self, window: &dyn Window) {
        window.request_redraw();
    }

    pub(super) fn push_pending_event_and_request_redraw(
        &mut self,
        window: &dyn Window,
        event: Event,
    ) {
        self.pending_events.push(event);
        self.request_sink_redraw(window);
    }
}
