use super::*;
use fret_ui::managed_surface::ManagedSurfaceEventCx;

mod internal_drag;
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
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.take_viewport_capture(window, cancel.pointer_id)
                },
            );
            if let Some(capture) = viewport_capture {
                let position = cancel.position.unwrap_or(capture.last);
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit,
                    declarative_pixels_per_point(cx.app(), window),
                    cancel.pointer_id,
                    cancel.pointer_type,
                    position,
                    fret_core::ViewportInputKind::PointerCancel {
                        buttons: cancel.buttons,
                        modifiers: cancel.modifiers,
                        reason: cancel.reason,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let cleared_tab = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.take_tab_close(window, cancel.pointer_id).is_some()
                        || service
                            .take_pending_dock_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_pending_dock_tabs_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_divider_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            let cleared_floating = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service
                        .take_floating_close(window, cancel.pointer_id)
                        .is_some()
                        || service
                            .take_floating_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            if cleared_tab || cleared_floating {
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        _ => {}
    }
}
