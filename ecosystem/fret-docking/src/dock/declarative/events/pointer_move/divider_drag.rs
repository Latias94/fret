use super::*;

pub(super) fn handle_pointer_move_divider_drag<H: UiHost + 'static>(
    cx: &mut fret_ui::managed_surface::ManagedSurfaceEventCx<'_, '_, H>,
    window: AppWindowId,
    position: fret_core::Point,
    buttons: fret_core::MouseButtons,
    pointer_id: fret_core::PointerId,
) -> bool {
    let divider_drag = cx.app().with_global_mut(
        DeclarativeDockInteractionService::default,
        |service, _app| {
            if buttons.left {
                service.divider_drag(window, pointer_id)
            } else {
                service.take_divider_drag(window, pointer_id)
            }
        },
    );
    let Some(divider_drag) = divider_drag else {
        return false;
    };

    if !buttons.left {
        cx.release_pointer_capture();
        cx.request_redraw();
        cx.stop_propagation();
        return true;
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
                .workspace
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
                super::super::super::super::split_geometry::drag_update_adjacent_fractions(
                    divider_drag.handle.axis,
                    divider_drag.handle.bounds,
                    children_len,
                    &fractions_now,
                    divider_drag.handle.handle_ix,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &divider_drag.min_px,
                    divider_drag.handle.grab_offset,
                    position,
                )
            else {
                return false;
            };

            dock.workspace
                .graph
                .update_split_fractions(divider_drag.handle.split, next)
        });
    if changed {
        cx.invalidate_self(fret_ui::Invalidation::Layout);
        cx.request_redraw();
    }
    cx.stop_propagation();
    true
}
