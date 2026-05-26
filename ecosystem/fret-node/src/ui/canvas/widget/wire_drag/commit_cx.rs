use fret_core::{AppWindowId, Rect};

use super::super::low_level_adapter::{self, CanvasPointerCaptureReleaseCx};

pub(in super::super) trait WireCommitCx<H>:
    CanvasPointerCaptureReleaseCx<H>
{
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
    fn bounds(&self, last_bounds: Option<Rect>) -> Rect;
}

pub(in super::super) fn invalidate_commit_paint<H>(cx: &mut impl WireCommitCx<H>) {
    low_level_adapter::invalidate_canvas_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::canvas::widget::low_level_adapter::{CanvasPaintInvalidationCx, CanvasRedrawCx};

    #[derive(Default)]
    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        host: StubHost,
        released: bool,
        redraws: usize,
        paint_invalidations: usize,
    }

    impl WireCommitCx<StubHost> for StubCx {
        fn host(&mut self) -> &mut StubHost {
            &mut self.host
        }

        fn window(&self) -> Option<AppWindowId> {
            None
        }

        fn bounds(&self, last_bounds: Option<Rect>) -> Rect {
            last_bounds.unwrap_or_default()
        }
    }

    impl CanvasPointerCaptureReleaseCx<StubHost> for StubCx {
        fn release_pointer_capture(&mut self) {
            self.released = true;
        }
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

    #[test]
    fn invalidate_commit_paint_requests_redraw_and_paint_invalidation() {
        let mut cx = StubCx::default();

        invalidate_commit_paint(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.released);
    }
}
