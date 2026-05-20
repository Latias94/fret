pub(super) trait WidgetRedrawCx<H> {
    fn request_redraw(&mut self);
}

pub(super) trait WidgetPaintInvalidationCx<H>: WidgetRedrawCx<H> {
    fn invalidate_paint(&mut self);
}

pub(super) trait WidgetHandledCx<H>: WidgetPaintInvalidationCx<H> {
    fn stop_propagation(&mut self);
}

pub(super) fn invalidate_widget_paint<H>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    cx.request_redraw();
    cx.invalidate_paint();
}

pub(super) fn finish_widget_handled<H>(cx: &mut impl WidgetHandledCx<H>) {
    cx.stop_propagation();
    invalidate_widget_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        stopped: bool,
        redraws: usize,
        paint_invalidations: usize,
    }

    impl WidgetRedrawCx<StubHost> for StubCx {
        fn request_redraw(&mut self) {
            self.redraws += 1;
        }
    }

    impl WidgetPaintInvalidationCx<StubHost> for StubCx {
        fn invalidate_paint(&mut self) {
            self.paint_invalidations += 1;
        }
    }

    impl WidgetHandledCx<StubHost> for StubCx {
        fn stop_propagation(&mut self) {
            self.stopped = true;
        }
    }

    #[test]
    fn invalidate_widget_paint_requests_redraw_and_paint_invalidation() {
        let mut cx = StubCx::default();

        invalidate_widget_paint(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
        assert!(!cx.stopped);
    }

    #[test]
    fn finish_widget_handled_stops_propagation_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_widget_handled(&mut cx);

        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
