use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn emit_graph_callbacks(
        &mut self,
        committed: &GraphTransaction,
        changes: &NodeGraphChanges,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };

        callbacks.on_graph_commit(committed, changes);
        if !changes.nodes.is_empty() {
            callbacks.on_nodes_change(&changes.nodes);
        }
        if !changes.edges.is_empty() {
            callbacks.on_edges_change(&changes.edges);
        }
        for change in connection_changes_from_transaction(committed) {
            callbacks.on_connection_change(change);
            match change {
                crate::runtime::callbacks::ConnectionChange::Connected(conn) => {
                    callbacks.on_connect(conn)
                }
                crate::runtime::callbacks::ConnectionChange::Disconnected(conn) => {
                    callbacks.on_disconnect(conn)
                }
                crate::runtime::callbacks::ConnectionChange::Reconnected { edge, from, to } => {
                    callbacks.on_reconnect(edge, from, to);
                    callbacks.on_edge_update(edge, from, to);
                }
            }
        }

        let deleted = crate::runtime::callbacks::delete_changes_from_transaction(committed);
        if !deleted.nodes.is_empty() {
            callbacks.on_nodes_delete(&deleted.nodes);
        }
        if !deleted.edges.is_empty() {
            callbacks.on_edges_delete(&deleted.edges);
        }
        if !deleted.groups.is_empty() {
            callbacks.on_groups_delete(&deleted.groups);
        }
        if !deleted.sticky_notes.is_empty() {
            callbacks.on_sticky_notes_delete(&deleted.sticky_notes);
        }
        if !deleted.nodes.is_empty()
            || !deleted.edges.is_empty()
            || !deleted.groups.is_empty()
            || !deleted.sticky_notes.is_empty()
        {
            callbacks.on_delete(deleted);
        }
    }

    fn drag_kind_from_wire_drag_kind(kind: &WireDragKind) -> ConnectDragKind {
        match kind {
            WireDragKind::New { from, bundle } => ConnectDragKind::New {
                from: *from,
                bundle: bundle.clone(),
            },
            WireDragKind::Reconnect {
                edge,
                endpoint,
                fixed,
            } => ConnectDragKind::Reconnect {
                edge: *edge,
                endpoint: *endpoint,
                fixed: *fixed,
            },
            WireDragKind::ReconnectMany { edges } => ConnectDragKind::ReconnectMany {
                edges: edges.clone(),
            },
        }
    }

    pub(super) fn emit_connect_start(&mut self, snapshot: &ViewSnapshot, kind: &WireDragKind) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        let ev = ConnectStart {
            kind: Self::drag_kind_from_wire_drag_kind(kind),
            mode: snapshot.interaction.connection_mode,
        };
        callbacks.on_connect_start(ev.clone());
        if matches!(
            kind,
            WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
        ) {
            callbacks.on_reconnect_start(ev.clone());
            callbacks.on_edge_update_start(ev);
        }
    }

    pub(super) fn emit_connect_end(
        &mut self,
        mode: crate::interaction::NodeGraphConnectionMode,
        kind: &WireDragKind,
        target: Option<PortId>,
        outcome: ConnectEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        let ev = ConnectEnd {
            kind: Self::drag_kind_from_wire_drag_kind(kind),
            mode,
            target,
            outcome,
        };
        callbacks.on_connect_end(ev.clone());
        if matches!(
            kind,
            WireDragKind::Reconnect { .. } | WireDragKind::ReconnectMany { .. }
        ) {
            callbacks.on_reconnect_end(ev.clone());
            callbacks.on_edge_update_end(ev);
        }
    }

    pub(super) fn emit_move_start(&mut self, snapshot: &ViewSnapshot, kind: ViewportMoveKind) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_move_start(ViewportMoveStart {
            kind,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        });
    }

    pub(super) fn emit_move_end(
        &mut self,
        snapshot: &ViewSnapshot,
        kind: ViewportMoveKind,
        outcome: ViewportMoveEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_move_end(ViewportMoveEnd {
            kind,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
            outcome,
        });
    }

    pub(super) fn emit_node_drag_start(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag_start(NodeDragStart {
            primary,
            nodes: nodes.to_vec(),
        });
    }

    pub(super) fn emit_node_drag_end(
        &mut self,
        primary: GraphNodeId,
        nodes: &[GraphNodeId],
        outcome: NodeDragEndOutcome,
    ) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag_end(NodeDragEnd {
            primary,
            nodes: nodes.to_vec(),
            outcome,
        });
    }

    pub(super) fn emit_node_drag(&mut self, primary: GraphNodeId, nodes: &[GraphNodeId]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        callbacks.on_node_drag(primary, nodes);
    }

    pub(super) fn emit_view_callbacks(&mut self, changes: &[ViewChange]) {
        let Some(callbacks) = self.callbacks.as_mut() else {
            return;
        };
        if changes.is_empty() {
            return;
        }
        callbacks.on_view_change(changes);
        for change in changes {
            match change {
                ViewChange::Viewport { pan, zoom } => {
                    callbacks.on_viewport_change(*pan, *zoom);
                    callbacks.on_move(*pan, *zoom);
                }
                ViewChange::Selection {
                    nodes,
                    edges,
                    groups,
                } => callbacks.on_selection_change(crate::runtime::callbacks::SelectionChange {
                    nodes: nodes.clone(),
                    edges: edges.clone(),
                    groups: groups.clone(),
                }),
            }
        }
    }
}
