use super::*;

pub(super) fn handle_pointer_wheel_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::PointerEvent,
    window: AppWindowId,
) {
    let fret_core::PointerEvent::Wheel {
        position, delta, ..
    } = event
    else {
        return;
    };

    let bounds = cx.bounds();
    let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds) else {
        return;
    };
    let theme = cx.theme().snapshot();

    if let Some(menu) = declarative_tab_overflow_menu_for_window(cx.app(), window) {
        let (handled, next_menu) = declarative_handle_tab_overflow_menu_wheel(
            cx.app(),
            menu,
            &snapshot.layout_all,
            theme.clone(),
            *position,
            *delta,
        );
        if handled {
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.set_tab_overflow_menu(window, next_menu),
            );
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }
    }

    if let Some(next_scroll) = declarative_handle_tab_strip_wheel(
        cx.app(),
        window,
        &snapshot.layout_all,
        theme,
        *position,
        *delta,
    ) {
        declarative_sync_tab_scroll_for_window(
            cx.app(),
            window,
            &next_scroll,
            snapshot.layout_all.keys().copied(),
        );
        cx.request_redraw();
        cx.stop_propagation();
    }
}
