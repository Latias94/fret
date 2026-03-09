use super::*;

#[derive(Debug, Clone)]
pub(super) struct LeftPointerDownSnapshot {
    pub(super) interaction: crate::io::NodeGraphInteractionConfig,
    pub(super) base_selection: Vec<crate::core::NodeId>,
    pub(super) hit: Option<crate::core::NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LeftPointerDownOutcome {
    HitNode { capture_pointer: bool },
    Marquee,
    EmptySpaceClear,
    Idle,
}

impl LeftPointerDownOutcome {
    pub(super) fn capture_pointer(self) -> bool {
        matches!(
            self,
            Self::HitNode {
                capture_pointer: true,
            } | Self::Marquee
                | Self::EmptySpaceClear
        )
    }
}

pub(super) fn begin_pan_pointer_down_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    down: fret_ui::action::PointerDownCx,
) -> bool {
    let _ = host.models_mut().update(marquee, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    let _ = host.models_mut().update(drag, |state| {
        *state = Some(DragState {
            button: down.button,
            last_pos: down.position,
        });
    });
    true
}

pub(super) fn read_left_pointer_down_snapshot_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    derived_cache: &Model<DerivedGeometryCacheState>,
    hit_scratch: &Model<Vec<crate::core::NodeId>>,
    down: fret_ui::action::PointerDownCx,
    bounds: Rect,
) -> LeftPointerDownSnapshot {
    let (interaction, base_selection, node_click_distance_screen_px, view) = host
        .models_mut()
        .read(view_state, |state| {
            (
                state.interaction.clone(),
                state.selected_nodes.clone(),
                state.interaction.node_click_distance,
                view_from_state(state),
            )
        })
        .ok()
        .unwrap_or((Default::default(), Vec::new(), 6.0, PanZoom2D::default()));

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
                    down.position,
                    scratch,
                )
            })
            .ok()
            .flatten()
    } else {
        None
    };

    LeftPointerDownSnapshot {
        interaction,
        base_selection,
        hit,
    }
}

pub(super) fn begin_left_pointer_down_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
    hovered: &Model<Option<crate::core::NodeId>>,
    down: fret_ui::action::PointerDownCx,
    snapshot: &LeftPointerDownSnapshot,
) -> LeftPointerDownOutcome {
    let multi = snapshot
        .interaction
        .multi_selection_key
        .is_pressed(down.modifiers);
    let selection_box_armed = snapshot.interaction.selection_on_drag
        || snapshot
            .interaction
            .selection_key
            .is_pressed(down.modifiers);

    if let Some(hit) = snapshot.hit {
        let _ = host.models_mut().update(marquee, |state| *state = None);
        let _ = host.models_mut().update(node_drag, |state| *state = None);
        let _ = host
            .models_mut()
            .update(pending_selection, |state| *state = None);
        let _ = host
            .models_mut()
            .update(hovered, |state| *state = Some(hit));
        if snapshot.interaction.elements_selectable {
            let preview_nodes =
                build_click_selection_preview_nodes(&snapshot.base_selection, hit, multi);
            let _ = host.models_mut().update(pending_selection, |state| {
                *state = Some(PendingSelectionState {
                    nodes: preview_nodes.clone(),
                    clear_edges: false,
                    clear_groups: false,
                });
            });

            if snapshot.interaction.nodes_draggable && !multi {
                let _ = host.models_mut().update(node_drag, |state| {
                    *state = Some(NodeDragState {
                        start_screen: down.position,
                        current_screen: down.position,
                        phase: NodeDragPhase::Armed,
                        nodes_sorted: preview_nodes,
                    });
                });
            }
        }
        return LeftPointerDownOutcome::HitNode {
            capture_pointer: snapshot.interaction.elements_selectable,
        };
    }

    let _ = host.models_mut().update(hovered, |state| *state = None);
    let _ = host.models_mut().update(node_drag, |state| *state = None);
    let _ = host
        .models_mut()
        .update(pending_selection, |state| *state = None);

    if selection_box_armed && snapshot.interaction.elements_selectable {
        let base_selected_nodes: Arc<[crate::core::NodeId]> = if multi {
            Arc::from(snapshot.base_selection.clone().into_boxed_slice())
        } else {
            Arc::from([])
        };
        let preview_selected_nodes: Arc<[crate::core::NodeId]> = if multi {
            base_selected_nodes.clone()
        } else {
            Arc::from([])
        };
        let _ = host.models_mut().update(marquee, |state| {
            *state = Some(MarqueeDragState {
                start_screen: down.position,
                current_screen: down.position,
                active: false,
                toggle: multi,
                base_selected_nodes,
                preview_selected_nodes,
            });
        });
        return LeftPointerDownOutcome::Marquee;
    }

    if snapshot.interaction.elements_selectable && !multi {
        let _ = host.models_mut().update(pending_selection, |state| {
            *state = Some(PendingSelectionState {
                nodes: Arc::from([]),
                clear_edges: true,
                clear_groups: true,
            });
        });
        return LeftPointerDownOutcome::EmptySpaceClear;
    }

    LeftPointerDownOutcome::Idle
}
