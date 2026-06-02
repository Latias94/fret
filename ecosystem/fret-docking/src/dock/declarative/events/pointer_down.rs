use super::*;

pub(super) fn handle_pointer_down_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::PointerEvent,
    window: AppWindowId,
) {
    let fret_core::PointerEvent::Down {
        position,
        button,
        modifiers,
        click_count,
        pointer_id,
        pointer_type,
        ..
    } = event
    else {
        return;
    };

    let theme = cx.theme().snapshot();
    let bounds = cx.bounds();
    if let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds) {
        let menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
        if let Some(menu) = menu {
            let (handled, next_menu, effects) = declarative_handle_tab_overflow_menu_left_click(
                cx.app(),
                window,
                menu,
                &snapshot.layout_all,
                theme.clone(),
                *position,
            );
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.set_tab_overflow_menu(window, next_menu)
                },
            );
            for effect in effects {
                cx.push_effect(effect);
            }
            cx.request_redraw();
            if handled {
                cx.stop_propagation();
                return;
            }
        } else {
            let tab_widths = declarative_tab_widths_for_layout(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
            );
            let tab_scroll = declarative_tab_scroll_for_frame(
                cx.app(),
                window,
                theme.clone(),
                &snapshot.layout_all,
                &tab_widths,
                false,
            );
            if let Some(menu) = declarative_open_tab_overflow_menu(
                cx.app(),
                window,
                &snapshot.layout_all,
                &tab_scroll,
                theme.clone(),
                *position,
            ) {
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.set_tab_overflow_menu(window, Some(menu))
                    },
                );
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }
        }

        if let Some(floating) = declarative_hit_test_floating_close(&snapshot, *position) {
            let pressed = DeclarativePressedFloatingClose {
                pointer_id: *pointer_id,
                floating,
            };
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.begin_floating_close(window, pressed)
                },
            );
            cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                window,
                floating,
            }));
            cx.capture_pointer(cx.node());
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }
        if let Some((floating, grab_offset, start_rect)) =
            declarative_hit_test_floating_title_bar(&snapshot, *position)
            && *button == fret_core::MouseButton::Left
        {
            let drag = DeclarativeFloatingDrag {
                pointer_id: *pointer_id,
                floating,
                grab_offset,
                start_rect,
                start: *position,
                start_tick: cx.app().tick_id(),
                activated: false,
                dock_previews_enabled: false,
            };
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.begin_floating_drag(window, drag)
                },
            );
            cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                window,
                floating,
            }));
            cx.capture_pointer(cx.node());
            cx.set_cursor_icon(fret_core::CursorIcon::Default);
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if let Some((handle, min_px)) =
            declarative_split_handle_hit_for_position(cx.app(), &snapshot, *position)
            && *button == fret_core::MouseButton::Left
        {
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.begin_divider_drag(
                        window,
                        DeclarativeDividerDrag {
                            pointer_id: *pointer_id,
                            handle,
                            min_px,
                        },
                    );
                },
            );
            cx.capture_pointer(cx.node());
            cx.set_cursor_icon(declarative_split_handle_cursor(handle.axis));
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        if matches!(
            *button,
            fret_core::MouseButton::Left
                | fret_core::MouseButton::Right
                | fret_core::MouseButton::Middle
        ) && let Some(hit) =
            declarative_hit_test_active_viewport_panel(cx.app(), window, bounds, *position)
        {
            let pixels_per_point = declarative_pixels_per_point(cx.app(), window);
            if let Some(input) = viewport_input_from_hit(
                window,
                hit.clone(),
                pixels_per_point,
                *pointer_id,
                *pointer_type,
                *position,
                fret_core::ViewportInputKind::PointerDown {
                    button: *button,
                    modifiers: *modifiers,
                    click_count: *click_count,
                },
            ) {
                cx.push_effect(Effect::ViewportInput(input));
            }
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.begin_viewport_capture(
                        window,
                        ViewportCaptureState {
                            pointer_id: *pointer_id,
                            hit,
                            button: *button,
                            start: *position,
                            last: *position,
                            moved: false,
                        },
                    );
                },
            );
            cx.capture_pointer(cx.node());
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }
    }
    if let Some((tabs, index, panel)) =
        declarative_hit_test_tab_close(cx.app(), window, bounds, theme.clone(), *position)
        && *button == fret_core::MouseButton::Left
    {
        let pressed = DeclarativePressedTabClose {
            pointer_id: *pointer_id,
            tabs,
            index,
            panel,
            start: *position,
        };
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.begin_tab_close(window, pressed)
            },
        );
        cx.capture_pointer(cx.node());
        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    if let Some((_tabs, _index, panel, grab_offset)) =
        declarative_hit_test_tab_content(cx.app(), window, bounds, theme.clone(), *position)
        && *button == fret_core::MouseButton::Left
        && declarative_panel_drag_allowed(cx.app(), window, &panel)
    {
        let pending = DeclarativePendingDockDrag {
            pointer_id: *pointer_id,
            start: *position,
            panel,
            grab_offset,
            start_tick: cx.app().tick_id(),
        };
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.begin_pending_dock_drag(window, pending)
            },
        );
        cx.capture_pointer(cx.node());
        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    if let Some((tabs, grab_offset)) =
        declarative_hit_test_tab_bar_empty_space(cx.app(), window, bounds, theme, *position)
        && *button == fret_core::MouseButton::Left
        && declarative_tabs_group_drag_allowed(cx.app(), window, tabs)
    {
        let pending = DeclarativePendingDockTabsDrag {
            pointer_id: *pointer_id,
            start: *position,
            tabs,
            grab_offset,
            start_tick: cx.app().tick_id(),
        };
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.begin_pending_dock_tabs_drag(window, pending)
            },
        );
        cx.capture_pointer(cx.node());
        cx.request_redraw();
        cx.stop_propagation();
    }
}
