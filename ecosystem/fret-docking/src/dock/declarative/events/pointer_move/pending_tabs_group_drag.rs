use super::*;

pub(super) fn handle_pointer_move_pending_tabs_group_drag<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    window: AppWindowId,
    position: fret_core::Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    pointer_id: fret_core::PointerId,
) -> bool {
    let pending = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            if buttons.left {
                service.pending_dock_tabs_drag(window, pointer_id)
            } else {
                service.take_pending_dock_tabs_drag(window, pointer_id)
            }
        },
    );
    let Some(pending) = pending else {
        return false;
    };

    if !buttons.left {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.stop_propagation();
        return true;
    }

    let settings = cx
        .app()
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let activation = fret_dnd::ActivationConstraint::Distance {
        px: settings.tab_drag_threshold.0,
    };
    if activation.is_satisfied(
        pending.start_tick.0,
        cx.app().tick_id().0,
        pending.start,
        position,
    ) {
        let pending = cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service, _app| service.take_pending_dock_tabs_drag(window, pointer_id),
        );
        if let Some(pending) = pending {
            begin_declarative_tabs_group_drag(
                cx.app(),
                window,
                pointer_id,
                pending,
                position,
                modifiers,
            );
            cx.app()
                .with_global_mut(DockManager::default, |dock, _app| {
                    dock.presentation.hover = None
                });
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return true;
        }
    }

    cx.request_redraw();
    cx.stop_propagation();
    true
}
