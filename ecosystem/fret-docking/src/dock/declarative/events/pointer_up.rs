use super::*;

pub(super) fn handle_pointer_up_event<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    event: &fret_core::PointerEvent,
    window: AppWindowId,
) {
    let fret_core::PointerEvent::Up {
        position,
        button,
        modifiers,
        is_click,
        click_count,
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
            service.pointer_up_owner(window, *pointer_id)
        },
    );
    if matches!(owner, DeclarativePointerUpOwner::ViewportCapture) {
        let Some(capture) = cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.take_viewport_capture(window, *pointer_id)
            },
        ) else {
            return;
        };
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
            .with_global_mut(DockManager::default, |dock, _app| {
                dock.presentation.hover = None
            });
        cx.release_pointer_capture();
        cx.request_redraw();
        if suppress_context_menu {
            cx.stop_propagation();
        }
        return;
    }

    if *button == fret_core::MouseButton::Left
        && matches!(owner, DeclarativePointerUpOwner::FloatingClose)
    {
        let pressed_floating = cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service: &mut DeclarativeDockInteractionService, _app| {
                service.take_floating_close(window, *pointer_id)
            },
        );
        if let Some(pressed) = pressed_floating {
            let bounds = cx.bounds();
            let clicked = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
                .and_then(|snapshot| declarative_hit_test_floating_close(&snapshot, *position))
                == Some(pressed.floating);
            if clicked
                && let Some(target_tabs) = cx
                    .app()
                    .global::<DockManager>()
                    .and_then(|dock| dock.workspace.graph.first_tabs_in_window(window))
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

    if *button == fret_core::MouseButton::Left
        && matches!(owner, DeclarativePointerUpOwner::FloatingDrag)
    {
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
                .with_global_mut(DockManager::default, |dock, _app| {
                    dock.presentation.hover = None
                });
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }
    }

    if *button == fret_core::MouseButton::Left
        && matches!(owner, DeclarativePointerUpOwner::DividerDrag)
    {
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
                    dock.workspace
                        .graph
                        .node(divider_drag.handle.split)
                        .and_then(|node| match node {
                            fret_core::DockNode::Split {
                                children,
                                fractions,
                                ..
                            } if children.len() >= 2 && children.len() == fractions.len() => {
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

    if *button == fret_core::MouseButton::Left
        && matches!(owner, DeclarativePointerUpOwner::PendingPanelDrag)
    {
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

    if *button == fret_core::MouseButton::Left
        && matches!(owner, DeclarativePointerUpOwner::PendingTabsGroupDrag)
    {
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

    if !matches!(owner, DeclarativePointerUpOwner::TabClose) {
        return;
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
    let clicked = declarative_hit_test_tab_close(cx.app(), window, bounds, theme, *position)
        .is_some_and(|(tabs, index, panel)| {
            tabs == pressed.tabs && index == pressed.index && panel == pressed.panel
        });
    let within_slop = fret_ui_headless::tab_strip_hit_test::pointer_move_within_slop(
        pressed.start,
        *position,
        super::super::super::consts::DOCK_TAB_CLOSE_CLICK_SLOP,
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
