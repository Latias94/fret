use fret_core::NodeId;
use fret_ui::{Invalidation, UiHost, retained_bridge::*};

pub(crate) fn request_paint_repaint<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}

pub(crate) fn finish_paint_event<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.stop_propagation();
    request_paint_repaint(cx);
}

pub(crate) fn focus_canvas_and_finish_paint_event<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    canvas_node: NodeId,
) {
    cx.request_focus(canvas_node);
    finish_paint_event(cx);
}

pub(crate) fn focus_canvas_and_finish_layout_event<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    canvas_node: NodeId,
) {
    cx.request_focus(canvas_node);
    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Layout);
}

pub(crate) fn finish_portal_command<H: UiHost>(
    cx: &mut CommandCx<'_, H>,
    focus_canvas: Option<NodeId>,
) {
    if let Some(canvas_node) = focus_canvas {
        cx.request_focus(canvas_node);
    }
    cx.stop_propagation();
    cx.request_redraw();
}
