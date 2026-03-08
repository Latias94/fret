use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_down<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        button: MouseButton,
        modifiers: fret_core::Modifiers,
        click_count: u8,
        zoom: f32,
    ) {
        super::event_pointer_down_state::prepare_pointer_down_state(
            self, cx, snapshot, position, modifiers,
        );
        super::event_pointer_down_route::route_pointer_down(
            self,
            cx,
            snapshot,
            position,
            button,
            modifiers,
            click_count,
            zoom,
        );
    }
}
