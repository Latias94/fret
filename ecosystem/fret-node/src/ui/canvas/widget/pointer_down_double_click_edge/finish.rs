use super::super::low_level_adapter::{CanvasHandledCx, finish_canvas_handled};

pub(super) fn finish_double_click<H>(cx: &mut impl CanvasHandledCx<H>) {
    finish_canvas_handled(cx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::canvas::widget::low_level_adapter::{CanvasPaintInvalidationCx, CanvasRedrawCx};

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
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

    #[test]
    fn finish_double_click_stops_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_double_click(&mut cx);

        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
