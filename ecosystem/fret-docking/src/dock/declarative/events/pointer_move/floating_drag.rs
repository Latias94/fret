use super::*;

pub(super) fn handle_pointer_move_floating_drag<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    window: AppWindowId,
    position: fret_core::Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    pointer_id: fret_core::PointerId,
) -> bool {
    let drag = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.take_floating_drag(window, pointer_id),
    );
    let Some(mut drag) = drag else {
        return false;
    };

    if !buttons.left {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.stop_propagation();
        return true;
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
            position,
        ) {
            drag.activated = true;
            drag.dock_previews_enabled = settings.drag_inversion.wants_dock_previews(modifiers);
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
            position,
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
    true
}
