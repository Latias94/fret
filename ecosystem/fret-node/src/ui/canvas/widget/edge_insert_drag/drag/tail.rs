use super::super::super::widget_tail::{WidgetPaintInvalidationCx, invalidate_widget_paint};

pub(super) fn finish_edge_insert_drag_move<H>(cx: &mut impl WidgetPaintInvalidationCx<H>) {
    invalidate_widget_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::super::super::super::widget_tail::WidgetRedrawCx;
    use super::*;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
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

    #[test]
    fn finish_edge_insert_drag_move_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_edge_insert_drag_move(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
