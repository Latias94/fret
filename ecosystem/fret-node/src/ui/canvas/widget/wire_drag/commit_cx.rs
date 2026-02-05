use fret_core::{AppWindowId, Rect};
use fret_ui::UiHost;

pub(in super::super) trait WireCommitCx<H: UiHost> {
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self, last_bounds: Option<Rect>) -> Rect;
    fn release_pointer_capture(&mut self);
    fn request_redraw(&mut self);
    fn invalidate_paint(&mut self);
}

impl<'a, H: UiHost> WireCommitCx<H> for fret_ui::retained_bridge::EventCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, _last_bounds: Option<Rect>) -> Rect {
        self.bounds
    }

    fn release_pointer_capture(&mut self) {
        self.release_pointer_capture();
    }

    fn request_redraw(&mut self) {
        self.request_redraw();
    }

    fn invalidate_paint(&mut self) {
        self.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}

impl<'a, H: UiHost> WireCommitCx<H> for fret_ui::retained_bridge::CommandCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, last_bounds: Option<Rect>) -> Rect {
        last_bounds.unwrap_or_default()
    }

    fn release_pointer_capture(&mut self) {}

    fn request_redraw(&mut self) {
        self.request_redraw();
    }

    fn invalidate_paint(&mut self) {
        self.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }
}
