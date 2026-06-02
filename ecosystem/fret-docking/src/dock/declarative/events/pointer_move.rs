use super::*;

mod divider_drag;
mod floating_drag;
mod hover;
mod viewport_capture;

pub(super) fn handle_pointer_move_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::PointerEvent,
    window: AppWindowId,
) {
    let fret_core::PointerEvent::Move {
        position,
        buttons,
        modifiers,
        pointer_id,
        pointer_type,
        ..
    } = event
    else {
        return;
    };

    if viewport_capture::handle_pointer_move_viewport_capture(
        cx,
        window,
        *position,
        *buttons,
        *modifiers,
        *pointer_id,
        *pointer_type,
    ) {
        return;
    }

    let divider_drag_handled = divider_drag::handle_pointer_move_divider_drag(
        cx,
        window,
        *position,
        *buttons,
        *pointer_id,
    );
    if divider_drag_handled {
        return;
    }

    let floating_drag_handled = floating_drag::handle_pointer_move_floating_drag(
        cx,
        window,
        *position,
        *buttons,
        *modifiers,
        *pointer_id,
    );
    if floating_drag_handled {
        return;
    }

    let pending = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            if buttons.left {
                service.pending_dock_drag(window, *pointer_id)
            } else {
                service.take_pending_dock_drag(window, *pointer_id)
            }
        },
    );
    if let Some(pending) = pending {
        if !buttons.left {
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return;
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
            *position,
        ) {
            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_pending_dock_drag(window, *pointer_id),
            );
            if let Some(pending) = pending {
                begin_declarative_panel_drag(
                    cx.app(),
                    window,
                    *pointer_id,
                    pending,
                    *position,
                    *modifiers,
                );
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }
        }

        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    let pending = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            if buttons.left {
                service.pending_dock_tabs_drag(window, *pointer_id)
            } else {
                service.take_pending_dock_tabs_drag(window, *pointer_id)
            }
        },
    );
    if let Some(pending) = pending {
        if !buttons.left {
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return;
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
            *position,
        ) {
            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_pending_dock_tabs_drag(window, *pointer_id),
            );
            if let Some(pending) = pending {
                begin_declarative_tabs_group_drag(
                    cx.app(),
                    window,
                    *pointer_id,
                    pending,
                    *position,
                    *modifiers,
                );
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }
        }

        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    hover::update_pointer_move_hover(cx, window, *position);
}
