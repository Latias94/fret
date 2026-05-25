use super::super::super::low_level_adapter::{CanvasPaintInvalidationCx, invalidate_canvas_paint};

pub(super) fn finish_edge_insert_drag_move<H>(cx: &mut impl CanvasPaintInvalidationCx<H>) {
    invalidate_canvas_paint(cx);
}

#[cfg(test)]
mod tests {
    use super::super::super::super::low_level_adapter::CanvasRedrawCx;
    use super::*;

    struct StubHost;

    #[derive(Default)]
    struct StubCx {
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

    #[test]
    fn finish_edge_insert_drag_move_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_edge_insert_drag_move(&mut cx);

        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
