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

    let owner = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service: &mut DeclarativeDockInteractionService, _app| {
            service.pointer_move_owner(window, *pointer_id)
        },
    );
    match owner {
        DeclarativePointerMoveOwner::ViewportCapture => {
            viewport_capture::handle_pointer_move_viewport_capture(
                cx,
                window,
                *position,
                *buttons,
                *modifiers,
                *pointer_id,
                *pointer_type,
            );
        }
        DeclarativePointerMoveOwner::BlockedByViewportCapture => {}
        DeclarativePointerMoveOwner::DividerDrag => {
            divider_drag::handle_pointer_move_divider_drag(
                cx,
                window,
                *position,
                *buttons,
                *pointer_id,
            );
        }
        DeclarativePointerMoveOwner::FloatingDrag => {
            floating_drag::handle_pointer_move_floating_drag(
                cx,
                window,
                *position,
                *buttons,
                *modifiers,
                *pointer_id,
            );
        }
        DeclarativePointerMoveOwner::PendingPanelDrag => {
            pending_panel_drag::handle_pointer_move_pending_panel_drag(
                cx,
                window,
                *position,
                *buttons,
                *modifiers,
                *pointer_id,
            );
        }
        DeclarativePointerMoveOwner::PendingTabsGroupDrag => {
            pending_tabs_group_drag::handle_pointer_move_pending_tabs_group_drag(
                cx,
                window,
                *position,
                *buttons,
                *modifiers,
                *pointer_id,
            );
        }
        DeclarativePointerMoveOwner::Hover => {
            hover::update_pointer_move_hover(cx, window, *position);
        }
    }
}
