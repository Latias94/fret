mod move_event;
mod wheel_event;

use fret_core::Modifiers;
use fret_ui::UiHost;

use super::*;

pub(super) fn handle_searcher_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    move_event::handle_searcher_pointer_move_event(canvas, cx, position, zoom)
}

pub(super) fn handle_searcher_wheel_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    delta: Point,
    modifiers: Modifiers,
    _zoom: f32,
) -> bool {
    wheel_event::handle_searcher_wheel_event(canvas, cx, delta, modifiers)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn update_searcher_hover_from_position(
        &mut self,
        position: Point,
        zoom: f32,
    ) -> bool {
        super::searcher_pointer_hover::update_searcher_hover_from_position(self, position, zoom)
    }

    pub(super) fn scroll_searcher_from_wheel(
        &mut self,
        delta: Point,
        modifiers: Modifiers,
    ) -> bool {
        super::searcher_pointer_wheel::scroll_searcher_from_wheel(self, delta, modifiers)
    }
}
