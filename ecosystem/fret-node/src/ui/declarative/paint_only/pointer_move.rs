use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct NodeDragPointerMoveOutcome {
    pub(super) capture_pointer: bool,
    pub(super) needs_layout_redraw: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MarqueePointerMoveOutcome {
    ReleaseCaptureRedrawOnly,
    NotifyRedraw,
}

pub(super) fn handle_node_drag_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    binding: &NodeGraphSurfaceBinding,
    mv: fret_ui::action::PointerMoveCx,
) -> Option<NodeDragPointerMoveOutcome> {
    let view_state = binding.view_state_model();
    let node_drag_value = host
        .models_mut()
        .read(node_drag, |state| state.clone())
        .ok()
        .flatten()?;

    if !mouse_buttons_contains(mv.buttons, MouseButton::Left) {
        return Some(NodeDragPointerMoveOutcome {
            capture_pointer: false,
            needs_layout_redraw: false,
        });
    }

    if node_drag_value.is_canceled() {
        let _ = host.models_mut().update(hovered, |state| *state = None);
        return Some(NodeDragPointerMoveOutcome {
            capture_pointer: false,
            needs_layout_redraw: false,
        });
    }

    let interaction = host
        .models_mut()
        .read(&view_state, |state| state.interaction.clone())
        .ok()
        .unwrap_or_default();
    let should_activate = pointer_crossed_threshold(
        node_drag_value.start_screen,
        mv.position,
        interaction.node_drag_threshold,
    );
    let capture_pointer = should_activate && node_drag_value.is_armed();

    if capture_pointer {
        let pending_selection_value = host
            .models_mut()
            .read(pending_selection, |state| state.clone())
            .ok()
            .flatten();
        if let Some(pending_selection_value) = pending_selection_value.as_ref() {
            let _ = commit_pending_selection_action_host(host, binding, pending_selection_value);
            let _ = host
                .models_mut()
                .update(pending_selection, |state| *state = None);
        }
    }

    let mut needs_layout_redraw = false;
    let _ = host.models_mut().update(node_drag, |state| {
        if let Some(state) = state.as_mut() {
            if should_activate && state.activate(mv.position) {
                needs_layout_redraw = true;
            }
            if state.update_active_position(mv.position) {
                needs_layout_redraw = true;
            }
        }
    });
    let _ = host.models_mut().update(hovered, |state| *state = None);

    Some(NodeDragPointerMoveOutcome {
        capture_pointer,
        needs_layout_redraw,
    })
}

pub(super) fn handle_marquee_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    mv: fret_ui::action::PointerMoveCx,
    bounds: Rect,
) -> Option<MarqueePointerMoveOutcome> {
    let marquee_value = host
        .models_mut()
        .read(marquee, |state| state.clone())
        .ok()
        .flatten()?;
    let (interaction, view) = host
        .models_mut()
        .read(view_state, |state| {
            (state.interaction.clone(), view_from_state(state))
        })
        .ok()
        .unwrap_or((Default::default(), PanZoom2D::default()));

    if !interaction.elements_selectable {
        let _ = host.models_mut().update(marquee, |state| *state = None);
        return Some(MarqueePointerMoveOutcome::ReleaseCaptureRedrawOnly);
    }

    let should_activate = pointer_crossed_threshold(
        marquee_value.start_screen,
        mv.position,
        interaction.node_click_distance,
    );
    let active_now = marquee_value.active || should_activate;

    let _ = host.models_mut().update(marquee, |state| {
        if let Some(state) = state.as_mut() {
            if should_activate {
                state.active = true;
            }
            if state.active {
                state.current_screen = mv.position;
            }
        }
    });

    if active_now {
        let (geom, index) = host
            .models_mut()
            .read(derived_cache, |state| {
                (state.geom.clone(), state.index.clone())
            })
            .ok()
            .unwrap_or((None, None));

        if let (Some(geom), Some(index)) = (geom.as_deref(), index.as_deref()) {
            let start_canvas = view.screen_to_canvas(bounds, marquee_value.start_screen);
            let cur_canvas = view.screen_to_canvas(bounds, mv.position);
            let rect_canvas = rect_from_points(start_canvas, cur_canvas);
            let preview_selected_nodes = build_marquee_preview_selected_nodes(
                &marquee_value,
                rect_canvas,
                interaction.selection_mode,
                geom,
                index,
            );
            let _ = host.models_mut().update(marquee, |state| {
                if let Some(state) = state.as_mut() {
                    state.preview_selected_nodes = preview_selected_nodes.clone();
                }
            });
        }
    }

    let _ = host.models_mut().update(hovered, |state| *state = None);
    Some(MarqueePointerMoveOutcome::NotifyRedraw)
}

pub(super) fn update_hovered_node_pointer_move_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    hovered: &Model<Option<crate::core::NodeId>>,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    hit_scratch: &Model<Vec<crate::core::NodeId>>,
    mv: fret_ui::action::PointerMoveCx,
    bounds: Rect,
) -> bool {
    let node_click_distance_screen_px = host
        .models_mut()
        .read(view_state, |state| state.interaction.node_click_distance)
        .ok()
        .unwrap_or(6.0);
    let view = host
        .models_mut()
        .read(view_state, view_from_state)
        .ok()
        .unwrap_or_default();

    let (geom, index) = host
        .models_mut()
        .read(derived_cache, |state| {
            (state.geom.clone(), state.index.clone())
        })
        .ok()
        .unwrap_or((None, None));

    let hit = if let (Some(geom), Some(index)) = (geom.as_deref(), index.as_deref()) {
        host.models_mut()
            .update(hit_scratch, |scratch| {
                hit_test_node_at_point(
                    view,
                    bounds,
                    node_click_distance_screen_px,
                    geom,
                    index,
                    mv.position,
                    scratch,
                )
            })
            .ok()
            .flatten()
    } else {
        None
    };

    host.models_mut()
        .update(hovered, |state| {
            if *state == hit {
                false
            } else {
                *state = hit;
                true
            }
        })
        .ok()
        .unwrap_or(false)
}
