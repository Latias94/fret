use super::*;
use fret_ui::managed_surface::ManagedSurfaceEventCx;

mod internal_drag;

pub(super) fn handle_declarative_event<H: UiHost + 'static>(
    cx: &mut ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::Event,
    window: AppWindowId,
    allow_multi_window_tear_off: bool,
) {
    match event {
        fret_core::Event::InternalDrag(e) => {
            internal_drag::handle_internal_drag_event(
                cx,
                event,
                e,
                window,
                allow_multi_window_tear_off,
            );
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            if let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            {
                let menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
                if let Some(menu) = menu {
                    let (handled, next_menu, effects) =
                        declarative_handle_tab_overflow_menu_left_click(
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
        fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            is_click,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.take_viewport_capture(window, *pointer_id)
                },
            );
            if let Some(capture) = viewport_capture {
                if capture.button != *button {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service: &mut DeclarativeDockInteractionService, _app| {
                            service.begin_viewport_capture(window, capture)
                        },
                    );
                    return;
                }
                let suppress_context_menu = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default()
                    .suppress_context_menu_during_viewport_capture
                    && capture.button == fret_core::MouseButton::Right
                    && capture.moved;
                let is_click = if suppress_context_menu {
                    false
                } else {
                    *is_click
                };
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit.clone(),
                    declarative_pixels_per_point(cx.app(), window),
                    *pointer_id,
                    *pointer_type,
                    *position,
                    fret_core::ViewportInputKind::PointerUp {
                        button: *button,
                        modifiers: *modifiers,
                        is_click,
                        click_count: *click_count,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                if suppress_context_menu {
                    cx.stop_propagation();
                }
                return;
            }

            if *button == fret_core::MouseButton::Left {
                let pressed_floating = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_floating_close(window, *pointer_id)
                    },
                );
                if let Some(pressed) = pressed_floating {
                    let bounds = cx.bounds();
                    let clicked =
                        declarative_layout_snapshot_for_bounds(cx.app(), window, bounds).and_then(
                            |snapshot| declarative_hit_test_floating_close(&snapshot, *position),
                        ) == Some(pressed.floating);
                    if clicked
                        && let Some(target_tabs) = cx
                            .app()
                            .global::<DockManager>()
                            .and_then(|dock| dock.graph.first_tabs_in_window(window))
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: pressed.floating,
                            target_tabs,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let floating_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_floating_drag(window, *pointer_id)
                    },
                );
                if let Some(drag) = floating_drag {
                    let bounds = cx.bounds();
                    let theme = cx.theme().snapshot();
                    if drag.activated
                        && drag.dock_previews_enabled
                        && let Some(DockDropTarget::Dock(target)) =
                            declarative_resolve_floating_title_bar_drag_target(
                                cx.app(),
                                window,
                                bounds,
                                theme,
                                drag.dock_previews_enabled,
                                *position,
                            )
                        && matches!(target.zone, fret_core::DropZone::Center)
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: drag.floating,
                            target_tabs: target.tabs,
                        }));
                    }
                    cx.app()
                        .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let divider_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_divider_drag(window, *pointer_id)
                    },
                );
                if let Some(divider_drag) = divider_drag {
                    let updates = cx
                        .app()
                        .with_global_mut(DockManager::default, |dock, _app| {
                            dock.graph
                                .node(divider_drag.handle.split)
                                .and_then(|node| match node {
                                    fret_core::DockNode::Split {
                                        children,
                                        fractions,
                                        ..
                                    } if children.len() >= 2
                                        && children.len() == fractions.len() =>
                                    {
                                        Some(vec![fret_core::SplitFractionsUpdate {
                                            split: divider_drag.handle.split,
                                            fractions: fractions.clone(),
                                        }])
                                    }
                                    _ => None,
                                })
                                .unwrap_or_default()
                        });
                    if !updates.is_empty() {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::SetSplitFractionsMany {
                            updates,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.invalidate_self(fret_ui::Invalidation::Layout);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_pending_dock_drag(window, *pointer_id)
                    },
                );
                if pending_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_tabs_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_pending_dock_tabs_drag(window, *pointer_id)
                    },
                );
                if pending_tabs_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            let pressed = if *button == fret_core::MouseButton::Left {
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service: &mut DeclarativeDockInteractionService, _app| {
                        service.take_tab_close(window, *pointer_id)
                    },
                )
            } else {
                None
            };
            let Some(pressed) = pressed else {
                return;
            };
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            let clicked =
                declarative_hit_test_tab_close(cx.app(), window, bounds, theme, *position)
                    .is_some_and(|(tabs, index, panel)| {
                        tabs == pressed.tabs && index == pressed.index && panel == pressed.panel
                    });
            let within_slop = fret_ui_headless::tab_strip_hit_test::pointer_move_within_slop(
                pressed.start,
                *position,
                super::super::consts::DOCK_TAB_CLOSE_CLICK_SLOP,
            );
            if clicked || within_slop {
                cx.push_effect(Effect::Dock(fret_core::DockOp::ClosePanel {
                    window,
                    panel: pressed.panel,
                }));
            }
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
        }
        fret_core::Event::PointerCancel(cancel) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service: &mut DeclarativeDockInteractionService, _app| {
                    service.take_viewport_capture(window, cancel.pointer_id)
                },
            );
            if let Some(capture) = viewport_capture {
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
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
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
        _ => {}
    }
}
