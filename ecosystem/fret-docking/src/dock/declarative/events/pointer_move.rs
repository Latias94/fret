use super::*;

mod divider_drag;
mod floating_drag;
mod hover;
mod pending_panel_drag;
mod pending_tabs_group_drag;
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

    let pending_panel_drag_handled = pending_panel_drag::handle_pointer_move_pending_panel_drag(
        cx,
        window,
        *position,
        *buttons,
        *modifiers,
        *pointer_id,
    );
    if pending_panel_drag_handled {
        return;
    }

    let pending_tabs_group_drag_handled =
        pending_tabs_group_drag::handle_pointer_move_pending_tabs_group_drag(
            cx,
            window,
            *position,
            *buttons,
            *modifiers,
            *pointer_id,
        );
    if pending_tabs_group_drag_handled {
        return;
    }

    hover::update_pointer_move_hover(cx, window, *position);
}
