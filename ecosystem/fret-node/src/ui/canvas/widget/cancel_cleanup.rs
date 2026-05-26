use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::interaction::NodeGraphConnectionMode;
use crate::runtime::callbacks::ConnectEndOutcome;

pub(super) fn cancel_cleanup_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    mode: NodeGraphConnectionMode,
) -> bool {
    let mut canceled = false;

    if let Some(wire_drag) = canvas.interaction.suspended_wire_drag.take() {
        canvas.emit_connect_end(mode, &wire_drag.kind, None, ConnectEndOutcome::Canceled);
        canceled = true;
    }
    canceled |= super::cancel_session::clear_cancel_residuals(&mut canvas.interaction);

    canceled
}

pub(super) fn clear_hover_and_focus<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    super::cancel_session::clear_hover_edge_focus(&mut canvas.interaction);
}

pub(super) fn finish_cancel<H>(
    cx: &mut impl super::low_level_adapter::HandledCanvasPointerCaptureReleaseCx<H>,
    consume: bool,
) {
    cx.release_pointer_capture();
    if consume {
        cx.stop_propagation();
    }
    super::low_level_adapter::invalidate_canvas_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::super::low_level_adapter::{
        CanvasHandledCx, CanvasPaintInvalidationCx, CanvasPointerCaptureReleaseCx, CanvasRedrawCx,
    };
    use super::*;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        released: bool,
        stopped: bool,
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
    fn finish_cancel_releases_and_invalidates_without_consuming() {
        let mut cx = StubCx::default();

        finish_cancel(&mut cx, false);

        assert!(cx.released);
        assert!(!cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }

    #[test]
    fn finish_cancel_releases_stops_and_invalidates_when_consuming() {
        let mut cx = StubCx::default();

        finish_cancel(&mut cx, true);

        assert!(cx.released);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
