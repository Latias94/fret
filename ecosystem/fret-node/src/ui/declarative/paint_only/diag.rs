use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclarativeDiagViewPreset {
    CenteredSelectionOnDrag,
    OffsetPartialMarquee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclarativeDiagKeyAction {
    NudgeVisibleNode,
    NormalizeVisibleNodeCentered,
    NormalizeVisibleNodeForMarquee,
    ArmFitToPortals,
    DisablePortals,
    EnablePortals,
    TogglePaintOverrides,
}

impl DeclarativeDiagKeyAction {
    pub(super) fn from_key(enabled: bool, key: fret_core::KeyCode) -> Option<Self> {
        if !enabled {
            return None;
        }

        match key {
            fret_core::KeyCode::Digit3 => Some(Self::NudgeVisibleNode),
            fret_core::KeyCode::Digit4 => Some(Self::NormalizeVisibleNodeCentered),
            fret_core::KeyCode::Digit5 => Some(Self::NormalizeVisibleNodeForMarquee),
            fret_core::KeyCode::Digit9 => Some(Self::ArmFitToPortals),
            fret_core::KeyCode::Digit8 => Some(Self::DisablePortals),
            fret_core::KeyCode::Digit7 => Some(Self::EnablePortals),
            fret_core::KeyCode::Digit6 => Some(Self::TogglePaintOverrides),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclarativeKeyboardZoomAction {
    ZoomIn,
    ZoomOut,
    Reset,
}

impl DeclarativeKeyboardZoomAction {
    pub(super) fn from_key(key: fret_core::KeyCode) -> Option<Self> {
        match key {
            fret_core::KeyCode::Equal
            | fret_core::KeyCode::NumpadAdd
            | fret_core::KeyCode::Digit1 => Some(Self::ZoomIn),
            fret_core::KeyCode::Minus
            | fret_core::KeyCode::NumpadSubtract
            | fret_core::KeyCode::Digit2 => Some(Self::ZoomOut),
            fret_core::KeyCode::Digit0 | fret_core::KeyCode::Numpad0 => Some(Self::Reset),
            _ => None,
        }
    }
}

pub(super) fn handle_declarative_escape_key_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    drag: &Model<Option<DragState>>,
    marquee: &Model<Option<MarqueeDragState>>,
    node_drag: &Model<Option<NodeDragState>>,
    pending_selection: &Model<Option<PendingSelectionState>>,
) -> bool {
    escape_cancel_declarative_interactions_action_host(
        host,
        drag,
        marquee,
        node_drag,
        pending_selection,
    )
}

fn commit_diag_graph_transaction_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
    build_tx: fn(&Graph) -> GraphTransaction,
) -> bool {
    let tx = host.models_mut().read(graph, build_tx).ok();
    if let Some(tx) = tx.as_ref() {
        let _ = commit_graph_transaction(host, graph, view_state, controller, tx);
    }
    true
}

pub(super) fn apply_declarative_diag_view_preset_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
    preset: DeclarativeDiagViewPreset,
) -> bool {
    update_view_state_action_host(host, view_state, controller, |state| {
        match preset {
            DeclarativeDiagViewPreset::CenteredSelectionOnDrag => {
                state.pan.x = 380.0;
                state.pan.y = 290.0;
                state.zoom = 1.0;
                state.interaction.selection_on_drag = true;
            }
            DeclarativeDiagViewPreset::OffsetPartialMarquee => {
                state.pan.x = 540.0;
                state.pan.y = 290.0;
                state.zoom = 1.0;
                state.interaction.selection_on_drag = true;
                state.interaction.selection_mode = crate::io::NodeGraphSelectionMode::Partial;
            }
        }
        state.selected_nodes.clear();
        state.selected_edges.clear();
        state.selected_groups.clear();
    })
}

fn toggle_diag_paint_overrides_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    graph: &Model<Graph>,
    diag_paint_overrides: &Arc<NodeGraphPaintOverridesMap>,
    diag_paint_overrides_enabled: &Model<bool>,
) -> bool {
    let enable_next = host
        .models_mut()
        .read(diag_paint_overrides_enabled, |state| !*state)
        .ok()
        .unwrap_or(true);
    let _ = host
        .models_mut()
        .update(diag_paint_overrides_enabled, |state| *state = enable_next);

    let edge_id = host
        .models_mut()
        .read(graph, |graph| graph.edges.keys().next().copied())
        .ok()
        .flatten();

    if let Some(edge_id) = edge_id {
        if enable_next {
            let mut stops = [GradientStop::new(0.0, Color::TRANSPARENT); MAX_STOPS];
            stops[0] = GradientStop::new(0.0, Color::from_srgb_hex_rgb(0xff_3b_30));
            stops[1] = GradientStop::new(1.0, Color::from_srgb_hex_rgb(0x34_c7_59));
            let gradient = LinearGradient {
                start: Point::new(Px(0.0), Px(0.0)),
                end: Point::new(Px(240.0), Px(0.0)),
                tile_mode: TileMode::Clamp,
                color_space: ColorSpace::Srgb,
                stop_count: 2,
                stops,
            };
            let paint = PaintBindingV1::with_eval_space(
                Paint::LinearGradient(gradient),
                PaintEvalSpaceV1::ViewportPx,
            );
            diag_paint_overrides.set_edge_override(
                edge_id,
                Some(
                    crate::ui::paint_overrides::EdgePaintOverrideV1 {
                        dash: Some(DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0))),
                        stroke_width_mul: Some(1.6),
                        stroke_paint: Some(paint),
                    }
                    .normalized(),
                ),
            );
        } else {
            diag_paint_overrides.set_edge_override(edge_id, None);
        }
    }

    true
}

pub(super) fn handle_declarative_diag_key_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    action: DeclarativeDiagKeyAction,
    graph: &Model<Graph>,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
    portal_bounds_store: &Model<PortalBoundsStore>,
    portal_debug_flags: &Model<PortalDebugFlags>,
    diag_paint_overrides: &Arc<NodeGraphPaintOverridesMap>,
    diag_paint_overrides_enabled: &Model<bool>,
) -> bool {
    match action {
        DeclarativeDiagKeyAction::NudgeVisibleNode => commit_diag_graph_transaction_action_host(
            host,
            graph,
            view_state,
            controller,
            build_diag_nudge_visible_node_transaction,
        ),
        DeclarativeDiagKeyAction::NormalizeVisibleNodeCentered => {
            commit_diag_graph_transaction_action_host(
                host,
                graph,
                view_state,
                controller,
                build_diag_normalize_visible_node_transaction,
            );
            let _ = apply_declarative_diag_view_preset_action_host(
                host,
                view_state,
                controller,
                DeclarativeDiagViewPreset::CenteredSelectionOnDrag,
            );
            true
        }
        DeclarativeDiagKeyAction::NormalizeVisibleNodeForMarquee => {
            commit_diag_graph_transaction_action_host(
                host,
                graph,
                view_state,
                controller,
                build_diag_normalize_visible_node_transaction,
            );
            let _ = apply_declarative_diag_view_preset_action_host(
                host,
                view_state,
                controller,
                DeclarativeDiagViewPreset::OffsetPartialMarquee,
            );
            true
        }
        DeclarativeDiagKeyAction::ArmFitToPortals => {
            let _ = host.models_mut().update(portal_bounds_store, |state| {
                state.pending_fit_to_portals = true;
            });
            true
        }
        DeclarativeDiagKeyAction::DisablePortals => {
            let _ = host.models_mut().update(portal_debug_flags, |state| {
                state.disable_portals = true;
            });
            let _ = host.models_mut().update(portal_bounds_store, |state| {
                state.nodes_canvas_bounds.clear();
                state.pending_fit_to_portals = false;
            });
            true
        }
        DeclarativeDiagKeyAction::EnablePortals => {
            let _ = host.models_mut().update(portal_debug_flags, |state| {
                state.disable_portals = false;
            });
            true
        }
        DeclarativeDiagKeyAction::TogglePaintOverrides => toggle_diag_paint_overrides_action_host(
            host,
            graph,
            diag_paint_overrides,
            diag_paint_overrides_enabled,
        ),
    }
}

pub(super) fn handle_declarative_keyboard_zoom_action_host(
    host: &mut dyn fret_ui::action::UiActionHost,
    action: DeclarativeKeyboardZoomAction,
    view_state: &Model<NodeGraphViewState>,
    controller: &NodeGraphController,
    min_zoom: f32,
    max_zoom: f32,
) -> bool {
    const KB_ZOOM_STEP_MUL: f32 = 1.1;
    update_view_state_action_host(host, view_state, controller, |state| {
        let zoom = PanZoom2D::sanitize_zoom(state.zoom, 1.0);
        state.zoom = match action {
            DeclarativeKeyboardZoomAction::ZoomIn => {
                (zoom * KB_ZOOM_STEP_MUL).clamp(min_zoom, max_zoom)
            }
            DeclarativeKeyboardZoomAction::ZoomOut => {
                (zoom * (1.0 / KB_ZOOM_STEP_MUL)).clamp(min_zoom, max_zoom)
            }
            DeclarativeKeyboardZoomAction::Reset => 1.0,
        };
    })
}
