use super::*;

mod hover;

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

    let viewport_capture = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.viewport_capture(window, *pointer_id),
    );
    if let Some(mut capture) = viewport_capture {
        capture.last = *position;
        if !capture.moved && capture.button == fret_core::MouseButton::Right {
            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let dx = position.x.0 - capture.start.x.0;
            let dy = position.y.0 - capture.start.y.0;
            let dist2 = dx * dx + dy * dy;
            let threshold = settings.viewport_context_menu_drag_threshold.0.max(0.0);
            capture.moved = dist2 >= threshold * threshold;
        }
        let input = viewport_input_from_hit_clamped(
            window,
            capture.hit.clone(),
            declarative_pixels_per_point(cx.app(), window),
            *pointer_id,
            *pointer_type,
            *position,
            fret_core::ViewportInputKind::PointerMove {
                buttons: *buttons,
                modifiers: *modifiers,
            },
        );
        cx.push_effect(Effect::ViewportInput(input));
        cx.app().with_global_mut(
            DeclarativeDockInteractionService::default,
            |service, _app| service.begin_viewport_capture(window, capture),
        );
        cx.request_redraw();
        cx.stop_propagation();
        return;
    }

    let viewport_capture_exists = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| service.has_viewport_capture_for_window(window),
    );
    if viewport_capture_exists {
        return;
    }

    let divider_drag = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            if buttons.left {
                service.divider_drag(window, *pointer_id)
            } else {
                service.take_divider_drag(window, *pointer_id)
            }
        },
    );
    if let Some(divider_drag) = divider_drag {
        if !buttons.left {
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
            return;
        }

        cx.set_cursor_icon(declarative_split_handle_cursor(divider_drag.handle.axis));
        let settings = cx
            .app()
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let changed = cx
            .app()
            .with_global_mut(DockManager::default, |dock, _app| {
                let Some((children_len, fractions_now)) = dock
                    .graph
                    .node(divider_drag.handle.split)
                    .and_then(|node| match node {
                        fret_core::DockNode::Split {
                            children,
                            fractions,
                            ..
                        } => Some((children.len(), fractions.clone())),
                        _ => None,
                    })
                else {
                    return false;
                };

                let Some(next) =
                    super::super::super::split_geometry::drag_update_adjacent_fractions(
                        divider_drag.handle.axis,
                        divider_drag.handle.bounds,
                        children_len,
                        &fractions_now,
                        divider_drag.handle.handle_ix,
                        settings.split_handle_gap,
                        settings.split_handle_hit_thickness,
                        &divider_drag.min_px,
                        divider_drag.handle.grab_offset,
                        *position,
                    )
                else {
                    return false;
                };

                dock.graph
                    .update_split_fractions(divider_drag.handle.split, next)
            });
        if changed {
            cx.invalidate_self(fret_ui::Invalidation::Layout);
            cx.request_redraw();
        }
        cx.stop_propagation();
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
