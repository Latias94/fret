use super::*;
use fret_ui::managed_surface::ManagedSurfaceEventCx;

mod internal_drag;
mod pointer_cancel;
mod pointer_down;
mod pointer_up;

pub(super) fn handle_declarative_event<H: UiHost + 'static>(
    cx: &mut ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::Event,
    window: AppWindowId,
    allow_multi_window_tear_off: bool,
) {
    match event {
        fret_core::Event::InternalDrag(e) => {
            internal_drag::handle_internal_drag_event(
                cx,
                event,
                e,
                window,
                allow_multi_window_tear_off,
            );
        }
        fret_core::Event::Pointer(event @ fret_core::PointerEvent::Down { .. }) => {
            pointer_down::handle_pointer_down_event(cx, event, window);
        }
        fret_core::Event::Pointer(event @ fret_core::PointerEvent::Up { .. }) => {
            pointer_up::handle_pointer_up_event(cx, event, window);
        }
        fret_core::Event::PointerCancel(cancel) => {
            pointer_cancel::handle_pointer_cancel_event(cx, cancel, window);
        }
        _ => {}
    }
}
