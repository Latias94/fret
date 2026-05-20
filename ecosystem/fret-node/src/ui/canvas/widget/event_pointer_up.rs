mod dispatch;
mod prelude;

use super::*;

pub(super) trait PointerUpRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    dispatch::PointerUpGuardCx<H, M> + pointer_up_cx::PointerUpCx<H, M>
{
}

impl<H, M, Cx> PointerUpRouteCx<H, M> for Cx
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: dispatch::PointerUpGuardCx<H, M> + pointer_up_cx::PointerUpCx<H, M>,
{
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_up<H: UiHost, Cx>(
        &mut self,
        cx: &mut Cx,
        snapshot: &ViewSnapshot,
        position: Point,
        button: MouseButton,
        click_count: u8,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) where
        Cx: PointerUpRouteCx<H, M>,
    {
        prelude::sync_pointer_up_modifier_state(self, snapshot, modifiers);

        if dispatch::handle_pointer_up_guards(self, cx, snapshot, position, button, zoom) {
            return;
        }

        let _ = pointer_up::handle_pointer_up(
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
