use super::*;

mod divider_drag;
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

    let drag = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.take_floating_drag(window, *pointer_id),
    );
    if let Some(mut drag) = drag {
        if !buttons.left {
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if !drag.activated {
            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let activation = fret_dnd::ActivationConstraint::Distance {
                px: settings.tab_drag_threshold.0,
            };
            if activation.is_satisfied(
                drag.start_tick.0,
                cx.app().tick_id().0,
                drag.start,
                *position,
            ) {
                drag.activated = true;
                drag.dock_previews_enabled =
                    settings.drag_inversion.wants_dock_previews(*modifiers);
            }
        }

        let desired = Rect::new(
            fret_core::Point::new(
                fret_core::Px(position.x.0 - drag.grab_offset.x.0),
                fret_core::Px(position.y.0 - drag.grab_offset.y.0),
            ),
            drag.start_rect.size,
        );
        let rect = clamp_declarative_floating_rect_to_bounds(desired, cx.bounds());
        cx.push_effect(Effect::Dock(fret_core::DockOp::SetFloatingRect {
            window,
            floating: drag.floating,
            rect,
        }));
        if drag.activated {
            let bounds = cx.bounds();
            let theme = cx.theme().snapshot();
            let hover = declarative_resolve_floating_title_bar_drag_target(
                cx.app(),
                window,
                bounds,
                theme,
                drag.dock_previews_enabled,
                *position,
            );
            cx.app()
                .with_global_mut(DockManager::default, |dock, _app| dock.hover = hover);
        }
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service, _app| service.begin_floating_drag(window, drag),
        );
        cx.set_cursor_icon(fret_core::CursorIcon::Default);
        cx.request_redraw();
        cx.stop_propagation();
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
