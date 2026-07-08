use super::*;

pub(super) fn handle_pointer_cancel_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    cancel: &fret_core::PointerCancelEvent,
    window: AppWindowId,
) {
    let owner = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service: &mut DeclarativeDockInteractionService, _app| {
            service.pointer_cancel_owner(window, cancel.pointer_id)
        },
    );
    if matches!(owner, DeclarativePointerCancelOwner::ViewportCapture) {
        let Some(capture) = cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.take_viewport_capture(window, cancel.pointer_id)
            },
        ) else {
            return;
        };
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
            .with_global_mut(DockManager::default, |dock, _app| {
                dock.presentation.hover = None
            });
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    if !matches!(
        owner,
        DeclarativePointerCancelOwner::ActiveDockingOrFloatingSession
    ) {
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
