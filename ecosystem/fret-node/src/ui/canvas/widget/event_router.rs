use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_event<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        event: &Event,
        snapshot: &ViewSnapshot,
    ) {
        let zoom = snapshot.zoom;

        match event {
            Event::ClipboardText { token, text } => {
                self.handle_clipboard_text(cx, *token, text);
            }
            Event::ClipboardTextUnavailable { token, .. } => {
                self.handle_clipboard_text_unavailable(cx, *token);
            }
            Event::WindowFocusChanged(false) => {
                if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
                    return;
                }

                cancel::handle_escape_cancel(self, cx);
                self.interaction.pan_activation_key_held = false;
                self.interaction.multi_selection_active = false;
                return;
            }
            Event::PointerCancel(_) => {
                cancel::cancel_active_gestures(self, cx);
                return;
            }
            Event::InternalDrag(e) => {
                if insert_node_drag::handle_internal_drag_event(self, cx, &snapshot, e, zoom) {
                    return;
                }
            }
            Event::Timer { token } => {
                self.handle_timer(cx, snapshot, *token);
            }
            Event::KeyDown { key, modifiers, .. } => {
                self.handle_key_down(cx, snapshot, *key, *modifiers);
            }
            Event::KeyUp { key, .. } => {
                self.handle_key_up(cx, snapshot, *key);
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                self.handle_pointer_down(
                    cx,
                    snapshot,
                    *position,
                    *button,
                    *modifiers,
                    *click_count,
                    zoom,
                );
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position,
                buttons,
                modifiers,
                ..
            }) => {
                self.handle_pointer_move(cx, snapshot, *position, *buttons, *modifiers, zoom);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                self.handle_pointer_up(
                    cx,
                    snapshot,
                    *position,
                    *button,
                    *click_count,
                    *modifiers,
                    zoom,
                );
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                self.handle_pointer_wheel(cx, snapshot, *position, *delta, *modifiers, zoom);
            }
            Event::Pointer(fret_core::PointerEvent::PinchGesture {
                position, delta, ..
            }) => {
                self.handle_pinch_gesture(cx, snapshot, *position, *delta);
            }
            _ => {}
        }
    }
}
