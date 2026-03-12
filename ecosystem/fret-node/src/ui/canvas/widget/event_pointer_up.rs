mod dispatch;
mod prelude;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_up<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        button: MouseButton,
        click_count: u8,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        prelude::sync_pointer_up_modifier_state(self, snapshot, modifiers);

        if dispatch::handle_pointer_up_guards(self, cx, snapshot, position, button, zoom) {
            return;
        }

        let _ = dispatch::dispatch_pointer_up(
            self,
            cx,
            snapshot,
            position,
            button,
            click_count,
            modifiers,
            zoom,
        );
    }
}
