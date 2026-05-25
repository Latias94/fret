use super::super::low_level_adapter::{
    CanvasHandledCx, CanvasPaintInvalidationCx, HandledCanvasPointerCaptureReleaseCx,
};
use super::super::*;

pub(super) fn dismiss_searcher_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl HandledCanvasPointerCaptureReleaseCx<H>,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    canvas.dismiss_searcher_overlay(cx);
    finish_searcher_event(cx)
}

pub(super) fn handle_searcher_escape_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl HandledCanvasPointerCaptureReleaseCx<H>,
) -> bool {
    dismiss_searcher_event(canvas, cx)
}

pub(super) fn invalidate_searcher_paint<H>(cx: &mut impl CanvasPaintInvalidationCx<H>) {
    super::super::low_level_adapter::invalidate_canvas_paint(cx);
}

pub(super) fn finish_searcher_event<H>(cx: &mut impl CanvasHandledCx<H>) -> bool {
    super::super::low_level_adapter::finish_canvas_handled(cx);
    true
}

#[cfg(test)]
mod tests {
    use super::super::super::low_level_adapter::{
        CanvasPaintInvalidationCx, CanvasPointerCaptureReleaseCx, CanvasRedrawCx,
    };
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
    fn invalidate_searcher_paint_requests_redraw_and_paint_invalidation() {
        let mut cx = StubCx::default();

        invalidate_searcher_paint(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.stopped);
        assert!(!cx.released);
    }

    #[test]
    fn finish_searcher_event_stops_and_invalidates_paint() {
        let mut cx = StubCx::default();

        assert!(finish_searcher_event(&mut cx));

        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.released);
    }
}
