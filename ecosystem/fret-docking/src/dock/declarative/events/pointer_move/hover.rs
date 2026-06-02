use super::*;

pub(super) fn update_pointer_move_hover<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    window: AppWindowId,
    position: fret_core::Point,
) {
    let bounds = cx.bounds();
    let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds) else {
        return;
    };
    let split_cursor = declarative_split_handle_hit_for_position(cx.app(), &snapshot, position)
        .map(|(handle, _min_px)| declarative_split_handle_cursor(handle.axis));
    let floating_close = declarative_hit_test_floating_close(&snapshot, position);
    let floating_title_bar = declarative_hit_test_floating_title_bar(&snapshot, position)
        .map(|(floating, _grab_offset, _rect)| floating);
    let floating_hover = DeclarativeFloatingHover {
        close: floating_close,
        title_bar: floating_title_bar,
    };
    let theme = cx.theme().snapshot();
    let (hover, next_menu, pointer_cursor) =
        declarative_tab_hover_for_position(cx.app(), window, &snapshot.layout_all, theme, position);
    let changed = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            let floating_hover_changed = service.set_floating_hover(window, floating_hover);
            let hover_changed = service.set_tab_hover(window, hover);
            let menu_changed = !service.tab_overflow_menu_matches(window, &next_menu);
            service.set_tab_overflow_menu(window, next_menu);
            floating_hover_changed || hover_changed || menu_changed
        },
    );
    if let Some(cursor) = split_cursor {
        cx.set_cursor_icon(cursor);
    } else if floating_close.is_some() {
        cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
    } else if floating_title_bar.is_some() {
        cx.set_cursor_icon(fret_core::CursorIcon::Default);
    } else if pointer_cursor {
        cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
    }
    if changed {
        cx.request_redraw();
    }
}
