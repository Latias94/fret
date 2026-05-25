//! Low-level canvas adapter contract for host operations shared by retained compatibility code.
//!
//! This module is intentionally retained-context agnostic. Concrete bindings to `fret_ui`
//! retained contexts live in `retained_low_level_adapter.rs`.

pub(super) trait CanvasRedrawCx<H> {
    fn request_redraw(&mut self);
}

pub(super) trait CanvasPaintInvalidationCx<H>: CanvasRedrawCx<H> {
    fn invalidate_paint(&mut self);
}

pub(super) trait CanvasHandledCx<H>: CanvasPaintInvalidationCx<H> {
    fn stop_propagation(&mut self);
}

pub(super) trait CanvasPointerCaptureReleaseCx<H>: CanvasPaintInvalidationCx<H> {
    fn release_pointer_capture(&mut self);
}

pub(super) trait HandledCanvasPointerCaptureReleaseCx<H>:
    CanvasPointerCaptureReleaseCx<H> + CanvasHandledCx<H>
{
}

impl<H, T> HandledCanvasPointerCaptureReleaseCx<H> for T where
    T: CanvasPointerCaptureReleaseCx<H> + CanvasHandledCx<H>
{
}

pub(super) fn invalidate_canvas_paint<H>(cx: &mut impl CanvasPaintInvalidationCx<H>) {
    cx.request_redraw();
    cx.invalidate_paint();
}

pub(super) fn finish_canvas_handled<H>(cx: &mut impl CanvasHandledCx<H>) {
    cx.stop_propagation();
    invalidate_canvas_paint(cx);
}

pub(super) fn finish_canvas_pointer_capture_release<H>(
    cx: &mut impl CanvasPointerCaptureReleaseCx<H>,
) {
    cx.release_pointer_capture();
    invalidate_canvas_paint(cx);
}

pub(super) fn finish_handled_canvas_pointer_capture_release<H>(
    cx: &mut impl HandledCanvasPointerCaptureReleaseCx<H>,
) {
    cx.release_pointer_capture();
    cx.stop_propagation();
    invalidate_canvas_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        stopped: bool,
        released: bool,
        redraws: usize,
        paint_invalidations: usize,
    }

    impl CanvasRedrawCx<StubHost> for StubCx {
        fn request_redraw(&mut self) {
            self.redraws += 1;
        }
    }

    impl CanvasPaintInvalidationCx<StubHost> for StubCx {
        fn invalidate_paint(&mut self) {
            self.paint_invalidations += 1;
        }
    }

    impl CanvasHandledCx<StubHost> for StubCx {
        fn stop_propagation(&mut self) {
            self.stopped = true;
        }
    }

    impl CanvasPointerCaptureReleaseCx<StubHost> for StubCx {
        fn release_pointer_capture(&mut self) {
            self.released = true;
        }
    }

    #[test]
    fn invalidate_canvas_paint_requests_redraw_and_paint_invalidation() {
        let mut cx = StubCx::default();

        invalidate_canvas_paint(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.stopped);
    }

    #[test]
    fn finish_canvas_handled_stops_propagation_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_canvas_handled(&mut cx);

        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn finish_canvas_pointer_capture_release_releases_capture_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_canvas_pointer_capture_release(&mut cx);

        assert!(cx.released);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.stopped);
    }

    #[test]
    fn finish_handled_canvas_pointer_capture_release_releases_stops_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_handled_canvas_pointer_capture_release(&mut cx);

        assert!(cx.released);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
