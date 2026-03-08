use super::*;

pub(super) fn route_non_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    match event {
        Event::ClipboardText { token, text } => {
            canvas.handle_clipboard_text(cx, *token, text);
            true
        }
        Event::ClipboardTextUnavailable { token, .. } => {
            canvas.handle_clipboard_text_unavailable(cx, *token);
            true
        }
        Event::WindowFocusChanged(false) => {
            if canvas.interaction.searcher.is_some() || canvas.interaction.context_menu.is_some() {
                return true;
            }

            cancel::handle_escape_cancel(canvas, cx);
            canvas.interaction.pan_activation_key_held = false;
            canvas.interaction.multi_selection_active = false;
            true
        }
        Event::PointerCancel(_) => {
            cancel::cancel_active_gestures(canvas, cx);
            true
        }
        Event::InternalDrag(e) => {
            insert_node_drag::handle_internal_drag_event(canvas, cx, snapshot, e, zoom)
        }
        Event::Timer { token } => {
            canvas.handle_timer(cx, snapshot, *token);
            true
        }
        Event::KeyDown { key, modifiers, .. } => {
            canvas.handle_key_down(cx, snapshot, *key, *modifiers);
            true
        }
        Event::KeyUp { key, .. } => {
            canvas.handle_key_up(cx, snapshot, *key);
            true
        }
        _ => false,
    }
}
