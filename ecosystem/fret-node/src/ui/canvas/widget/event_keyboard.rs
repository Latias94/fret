use super::*;

pub(in crate::ui::canvas::widget) trait KeyboardInputFocusCx {
    fn focus_is_text_input(&self) -> bool;
}

pub(in crate::ui::canvas::widget) trait KeyboardInputSink<H: UiHost, M: NodeGraphCanvasMiddleware>:
    super::event_keyboard_route::KeyboardRouteCx<H, M>
    + super::low_level_adapter::CanvasPaintInvalidationCx<H>
    + KeyboardInputFocusCx
{
}

impl<H: UiHost, M, T> KeyboardInputSink<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: super::event_keyboard_route::KeyboardRouteCx<H, M>
        + super::low_level_adapter::CanvasPaintInvalidationCx<H>
        + KeyboardInputFocusCx,
{
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_key_down<H: UiHost>(
        &mut self,
        cx: &mut impl KeyboardInputSink<H, M>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
        modifiers: fret_core::Modifiers,
    ) {
        if super::event_keyboard_state::should_ignore_key_down(cx.focus_is_text_input()) {
            return;
        }

        super::event_keyboard_state::sync_keyboard_modifier_state(self, snapshot, modifiers);
        super::event_keyboard_route::route_key_down(self, cx, snapshot, key, modifiers);
    }

    pub(super) fn handle_key_up<H: UiHost>(
        &mut self,
        cx: &mut impl super::low_level_adapter::CanvasPaintInvalidationCx<H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
    ) {
        super::event_keyboard_route::route_key_up(self, cx, snapshot, key);
    }
}
