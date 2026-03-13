use fret_core::{Modifiers, MouseButton, Point};
use fret_ui::UiHost;

use super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, searcher_activation, searcher_input,
    searcher_pointer, searcher_ui,
};

pub(super) fn handle_searcher_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    searcher_ui::handle_searcher_escape_event(canvas, cx)
}

pub(super) fn handle_searcher_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
    modifiers: Modifiers,
) -> bool {
    searcher_input::handle_searcher_key_down_event(canvas, cx, key, modifiers)
}

pub(super) fn handle_searcher_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    searcher_activation::handle_searcher_pointer_down_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    searcher_activation::handle_searcher_pointer_up_event(canvas, cx, position, button, zoom)
}

pub(super) fn handle_searcher_pointer_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    searcher_pointer::handle_searcher_pointer_move_event(canvas, cx, position, zoom)
}

pub(super) fn handle_searcher_wheel<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    delta: Point,
    modifiers: Modifiers,
    zoom: f32,
) -> bool {
    searcher_pointer::handle_searcher_wheel_event(canvas, cx, delta, modifiers, zoom)
}
