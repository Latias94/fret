use super::*;

pub(in crate::ui::canvas::widget) trait SystemLifecycleCx<H: UiHost>:
    super::event_clipboard::ClipboardTextCx<H>
    + super::cancel_cx::CancelGestureCx<H>
    + super::internal_drag_cx::InternalDragCx<H>
    + super::timer_motion_cx::TimerMotionCx<H>
{
}

impl<H: UiHost, T> SystemLifecycleCx<H> for T where
    T: super::event_clipboard::ClipboardTextCx<H>
        + super::cancel_cx::CancelGestureCx<H>
        + super::internal_drag_cx::InternalDragCx<H>
        + super::timer_motion_cx::TimerMotionCx<H>
{
}

pub(super) fn route_lifecycle_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SystemLifecycleCx<H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    match event {
        Event::ClipboardReadText { token, text } => {
            canvas.handle_clipboard_text(cx, *token, text);
            true
        }
        Event::ClipboardReadFailed { token, .. } => {
            canvas.handle_clipboard_text_unavailable(cx, *token);
            true
        }
        Event::WindowFocusChanged(false) => {
            if super::menu_session::has_active_menu_session(&canvas.interaction) {
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
        _ => false,
    }
}
