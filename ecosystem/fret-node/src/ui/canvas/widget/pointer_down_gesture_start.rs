mod close_button;
mod menu;
mod pan_start;
mod pending_right_click;
mod sticky;

use super::*;

pub(super) trait PointerDownStartCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    context_menu::ContextMenuCx<H, M>
    + cancel_cx::CancelGestureCx<H>
    + sticky_wire_targets::StickyWireTargetPickerCx<H>
    + pan_zoom_begin_cx::PanZoomBeginCx<H>
{
}

impl<H, M, T> PointerDownStartCx<H, M> for T
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    T: context_menu::ContextMenuCx<H, M>
        + cancel_cx::CancelGestureCx<H>
        + sticky_wire_targets::StickyWireTargetPickerCx<H>
        + pan_zoom_begin_cx::PanZoomBeginCx<H>,
{
}

pub(super) fn handle_close_button_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl pointer_down_close_button_cx::PointerDownCloseButtonCx<H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    close_button::handle_close_button_pointer_down(canvas, cx, snapshot, position, button, zoom)
}

pub(super) fn handle_context_menu_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl context_menu::ContextMenuCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    menu::handle_context_menu_pointer_down(canvas, cx, position, button, zoom)
}

pub(super) fn handle_pending_right_click_start<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerDownStartCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
) -> bool {
    pending_right_click::handle_pending_right_click_start(canvas, cx, snapshot, position, button)
}

pub(super) fn handle_sticky_wire_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerDownStartCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    sticky::handle_sticky_wire_pointer_down(canvas, cx, snapshot, position, button, zoom)
}

pub(super) fn handle_pan_start_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerDownStartCx<H, M>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
) -> bool {
    pan_start::handle_pan_start_pointer_down(canvas, cx, snapshot, position, button, modifiers)
}
