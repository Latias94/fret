use fret_core::{AppWindowId, Point};
use fret_ui::UiHost;

use super::super::low_level_adapter::{CanvasHandledCx, CanvasPointerCaptureReleaseCx};
use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::core::CanvasPoint;

pub(in super::super) trait StickyWireTargetPickerCx<H>:
    CanvasPointerCaptureReleaseCx<H> + CanvasHandledCx<H>
{
    fn host(&mut self) -> &mut H;
    fn window(&self) -> Option<AppWindowId>;
}

pub(super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl StickyWireTargetPickerCx<H>,
    edge_id: crate::core::EdgeId,
    position: Point,
) -> bool {
    let window = cx.window();
    canvas.open_edge_insert_node_picker(cx.host(), window, edge_id, position);
    finish_sticky_wire_target_picker(cx);
    true
}

pub(super) fn open_connection_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl StickyWireTargetPickerCx<H>,
    from_port: crate::core::PortId,
    at: CanvasPoint,
) -> bool {
    canvas.open_connection_insert_node_picker(cx.host(), from_port, at);
    finish_sticky_wire_target_picker(cx);
    true
}

fn finish_sticky_wire_target_picker<H>(
    cx: &mut impl super::super::low_level_adapter::CanvasHandledCx<H>,
) {
    super::super::low_level_adapter::finish_canvas_handled(cx);
}

#[cfg(test)]
mod tests {
    use super::super::super::low_level_adapter::{
        CanvasHandledCx, CanvasPaintInvalidationCx, CanvasPointerCaptureReleaseCx, CanvasRedrawCx,
    };
    use super::*;

    #[derive(Default)]
    struct StubHost;

    #[derive(Default)]
    struct StubCx {
        host: StubHost,
        window: Option<fret_core::AppWindowId>,
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

    impl StickyWireTargetPickerCx<StubHost> for StubCx {
        fn host(&mut self) -> &mut StubHost {
            &mut self.host
        }

        fn window(&self) -> Option<AppWindowId> {
            self.window
        }
    }

    #[test]
    fn finish_sticky_wire_target_picker_stops_and_invalidates_paint() {
        let mut cx = StubCx::default();

        finish_sticky_wire_target_picker(&mut cx);

        assert!(!cx.released);
        assert!(cx.stopped);
        assert_eq!(cx.redraws, 1);
        assert_eq!(cx.paint_invalidations, 1);
    }
}
