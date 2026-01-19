use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_command<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        command: &CommandId,
    ) -> bool {
        match command.as_str() {
            CMD_NODE_GRAPH_OPEN_INSERT_NODE => {
                let at = self
                    .interaction
                    .last_canvas_pos
                    .or_else(|| {
                        let bounds = self.interaction.last_bounds?;
                        let cx0 = bounds.origin.x.0 + 0.5 * bounds.size.width.0;
                        let cy0 = bounds.origin.y.0 + 0.5 * bounds.size.height.0;
                        let center = Point::new(Px(cx0), Px(cy0));
                        Some(Self::screen_to_canvas(
                            bounds,
                            center,
                            snapshot.pan,
                            snapshot.zoom,
                        ))
                    })
                    .unwrap_or_default();
                self.open_insert_node_picker(cx.app, at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_CREATE_GROUP => {
                let at = self.interaction.last_canvas_pos.unwrap_or_default();
                self.create_group_at(cx.app, cx.window, at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let groups = snapshot.selected_groups.clone();
                if groups.is_empty() {
                    return true;
                }
                self.update_view_state(cx.app, |s| {
                    let mut selected_in_order: Vec<crate::core::GroupId> = Vec::new();
                    for id in &s.group_draw_order {
                        if groups.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    for id in &groups {
                        if !selected_in_order.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    s.group_draw_order.retain(|id| !groups.contains(id));
                    s.group_draw_order.extend(selected_in_order);
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_SEND_TO_BACK => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let groups = snapshot.selected_groups.clone();
                if groups.is_empty() {
                    return true;
                }
                self.update_view_state(cx.app, |s| {
                    let mut selected_in_order: Vec<crate::core::GroupId> = Vec::new();
                    for id in &s.group_draw_order {
                        if groups.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    for id in &groups {
                        if !selected_in_order.contains(id) {
                            selected_in_order.push(*id);
                        }
                    }
                    s.group_draw_order.retain(|id| !groups.contains(id));
                    let mut next = selected_in_order;
                    next.extend_from_slice(&s.group_draw_order);
                    s.group_draw_order = next;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_GROUP_RENAME => {
                self.interaction.context_menu = None;
                self.interaction.searcher = None;
                let Some(overlays) = self.overlays.clone() else {
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "group rename overlay not configured",
                    );
                    return true;
                };
                let Some(group) = snapshot.selected_groups.last().copied() else {
                    return true;
                };
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                let _ = overlays.update(cx.app, |s, _cx| {
                    s.group_rename = Some(GroupRenameOverlay {
                        group,
                        invoked_at_window: invoked_at,
                    });
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE => {
                if snapshot.selected_edges.len() != 1 {
                    return true;
                }
                let edge = snapshot.selected_edges[0];
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                self.open_edge_insert_node_picker(cx.app, cx.window, edge, invoked_at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_INSERT_REROUTE => {
                if snapshot.selected_edges.len() != 1 {
                    return true;
                }
                let edge_id = snapshot.selected_edges[0];
                let invoked_at = self
                    .interaction
                    .last_pos
                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                let at = self.reroute_pos_for_invoked_at(invoked_at);

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter.plan_split_edge(
                                graph,
                                edge_id,
                                &NodeKindKey::new(REROUTE_KIND),
                                at,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Ok(plan.ops),
                                ConnectDecision::Reject => Err(plan.diagnostics),
                            }
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Reroute"), ops) {
                            if let Some(node_id) = node_id {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
                                    s.selected_nodes.clear();
                                    s.selected_nodes.push(node_id);
                                    s.draw_order.retain(|id| *id != node_id);
                                    s.draw_order.push(node_id);
                                });
                            }
                        }
                    }
                    Some(Err(diags)) => {
                        if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                            self.show_toast(cx.app, cx.window, sev, msg);
                        }
                    }
                    None => {}
                }

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER => {
                let Some(ctx0) = self.interaction.last_conversion.clone() else {
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "no recent conversion candidates",
                    );
                    return true;
                };

                let rows = super::super::searcher::build_rows_flat(&ctx0.candidates, "");
                let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let origin = self.clamp_searcher_origin(
                    Point::new(Px(ctx0.at.x), Px(ctx0.at.y)),
                    visible,
                    bounds,
                    &snapshot,
                );
                let active_row =
                    Self::searcher_first_selectable_row(&rows).min(rows.len().saturating_sub(1));

                self.interaction.context_menu = None;
                self.interaction.searcher = Some(SearcherState {
                    origin,
                    invoked_at: Point::new(Px(ctx0.at.x), Px(ctx0.at.y)),
                    target: ContextMenuTarget::ConnectionConvertPicker {
                        from: ctx0.from,
                        to: ctx0.to,
                        at: ctx0.at,
                    },
                    query: String::new(),
                    candidates: ctx0.candidates,
                    recent_kinds: self.interaction.recent_kinds.clone(),
                    rows,
                    hovered_row: None,
                    active_row,
                    scroll: 0,
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_FRAME_SELECTION => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let did =
                    self.frame_nodes_in_view(cx.app, cx.window, bounds, &snapshot.selected_nodes);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FRAME_ALL => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let nodes = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        graph.nodes.keys().copied().collect::<Vec<_>>()
                    })
                    .ok()
                    .unwrap_or_default();
                let did = self.frame_nodes_in_view(cx.app, cx.window, bounds, &nodes);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_RESET_VIEW => {
                self.update_view_state(cx.app, |s| {
                    s.pan = CanvasPoint::default();
                    s.zoom = 1.0;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ZOOM_IN => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                self.zoom_about_center_factor(bounds, 1.2);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ZOOM_OUT => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                self.zoom_about_center_factor(bounds, 1.0 / 1.2);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE => {
                let next = match snapshot.interaction.connection_mode {
                    NodeGraphConnectionMode::Strict => NodeGraphConnectionMode::Loose,
                    NodeGraphConnectionMode::Loose => NodeGraphConnectionMode::Strict,
                };

                self.update_view_state(cx.app, |s| {
                    s.interaction.connection_mode = next;
                });
                self.show_toast(
                    cx.app,
                    cx.window,
                    DiagnosticSeverity::Info,
                    match next {
                        NodeGraphConnectionMode::Strict => "connection mode: strict",
                        NodeGraphConnectionMode::Loose => "connection mode: loose",
                    },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_UNDO => {
                let did = self.undo_last(cx.app, cx.window);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_REDO => {
                let did = self.redo_last(cx.app, cx.window);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT => {
                let did = self.focus_next_node(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV => {
                let did = self.focus_next_node(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE => {
                let did = self.focus_next_edge(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV_EDGE => {
                let did = self.focus_next_edge(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_NEXT_PORT => {
                let did = self.focus_next_port(cx.app, true);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PREV_PORT => {
                let did = self.focus_next_port(cx.app, false);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_LEFT => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Left);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_RIGHT => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Right);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_UP => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Up);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_FOCUS_PORT_DOWN => {
                let did = self.focus_port_direction(cx.app, &snapshot, PortNavDir::Down);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_ACTIVATE => {
                let did = self.activate_focused_port(cx, &snapshot);
                if did {
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                true
            }
            CMD_NODE_GRAPH_SELECT_ALL => {
                if !snapshot.interaction.elements_selectable {
                    return true;
                }
                let (nodes, groups, edges) = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        let nodes = graph
                            .nodes
                            .keys()
                            .copied()
                            .filter(|id| {
                                Self::node_is_selectable(graph, &snapshot.interaction, *id)
                            })
                            .collect::<Vec<_>>();
                        let groups = graph.groups.keys().copied().collect::<Vec<_>>();
                        let edges = if snapshot.interaction.edges_selectable {
                            graph
                                .edges
                                .keys()
                                .copied()
                                .filter(|id| {
                                    Self::edge_is_selectable(graph, &snapshot.interaction, *id)
                                })
                                .collect::<Vec<_>>()
                        } else {
                            Vec::new()
                        };
                        (nodes, groups, edges)
                    })
                    .ok()
                    .unwrap_or_default();

                self.interaction.focused_edge = None;
                self.interaction.focused_node = None;
                self.interaction.focused_port = None;
                self.interaction.focused_port_valid = false;
                self.interaction.focused_port_convertible = false;
                self.update_view_state(cx.app, |s| {
                    s.selected_nodes = nodes;
                    s.selected_groups = groups;
                    s.selected_edges = edges;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_COPY => {
                self.copy_selection_to_clipboard(
                    cx.app,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );
                true
            }
            CMD_NODE_GRAPH_CUT => {
                self.copy_selection_to_clipboard(
                    cx.app,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );

                let selected_nodes = snapshot.selected_nodes.clone();
                let selected_edges = snapshot.selected_edges.clone();
                let selected_groups = snapshot.selected_groups.clone();
                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(
                            graph,
                            &snapshot.interaction,
                            &selected_nodes,
                            &selected_edges,
                            &selected_groups,
                        )
                    })
                    .ok()
                    .unwrap_or_default();
                if remove_ops.is_empty() {
                    return true;
                }
                let (removed_nodes, removed_edges, removed_groups) =
                    Self::removed_ids_from_ops(&remove_ops);
                let _ = self.commit_ops(cx.app, cx.window, Some("Cut"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| !removed_edges.contains(id));
                    s.selected_nodes.retain(|id| !removed_nodes.contains(id));
                    s.selected_groups.retain(|id| !removed_groups.contains(id));
                });

                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_PASTE => {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                let at = self.next_paste_canvas_point(bounds, &snapshot);
                self.request_paste_at_canvas(cx.app, cx.window, at);
                true
            }
            CMD_NODE_GRAPH_DUPLICATE => {
                self.duplicate_selection(
                    cx.app,
                    cx.window,
                    &snapshot.selected_nodes,
                    &snapshot.selected_groups,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DELETE_SELECTION => {
                let preferred_focus = self
                    .interaction
                    .focused_edge
                    .or_else(|| snapshot.selected_edges.first().copied());
                let selected_edges = snapshot.selected_edges.clone();
                let selected_nodes = snapshot.selected_nodes.clone();
                let selected_groups = snapshot.selected_groups.clone();
                if selected_edges.is_empty()
                    && selected_nodes.is_empty()
                    && selected_groups.is_empty()
                {
                    return true;
                }

                let remove_ops = self
                    .graph
                    .read_ref(cx.app, |graph| {
                        Self::delete_selection_ops(
                            graph,
                            &snapshot.interaction,
                            &selected_nodes,
                            &selected_edges,
                            &selected_groups,
                        )
                    })
                    .ok()
                    .unwrap_or_default();

                if remove_ops.is_empty() {
                    return true;
                }
                let (removed_nodes, removed_edges, removed_groups) =
                    Self::removed_ids_from_ops(&remove_ops);
                let _ = self.commit_ops(cx.app, cx.window, Some("Delete Selection"), remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| !removed_edges.contains(id));
                    s.selected_nodes.retain(|id| !removed_nodes.contains(id));
                    s.selected_groups.retain(|id| !removed_groups.contains(id));
                });
                self.repair_focused_edge_after_graph_change(cx.app, preferred_focus);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_LEFT => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: -1.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 1.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_UP => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: -1.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_DOWN => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: 1.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_LEFT_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: -10.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_RIGHT_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 10.0, y: 0.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_UP_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: -10.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_NUDGE_DOWN_FAST => {
                self.nudge_selection_by_screen_delta(
                    cx.app,
                    cx.window,
                    &snapshot,
                    CanvasPoint { x: 0.0, y: 10.0 },
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_LEFT => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignLeft,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_RIGHT => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignRight,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_TOP => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignTop,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_BOTTOM => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignBottom,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_CENTER_X => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignCenterX,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_ALIGN_CENTER_Y => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::AlignCenterY,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DISTRIBUTE_X => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::DistributeX,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            CMD_NODE_GRAPH_DISTRIBUTE_Y => {
                self.align_or_distribute_selection(
                    cx.app,
                    cx.window,
                    &snapshot,
                    AlignDistributeMode::DistributeY,
                );
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                true
            }
            _ => false,
        }
    }
}
