use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn focus_next_edge<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable
            || !snapshot.interaction.edges_selectable
            || !snapshot.interaction.edges_focusable
        {
            return false;
        }

        let mut edges: Vec<EdgeId> = self
            .graph
            .read_ref(host, |g| {
                g.edges
                    .keys()
                    .copied()
                    .filter(|id| Self::edge_is_selectable(g, &snapshot.interaction, *id))
                    .collect()
            })
            .ok()
            .unwrap_or_default();
        if edges.is_empty() {
            return false;
        }
        edges.sort_unstable();

        let current = self
            .interaction
            .focused_edge
            .or_else(|| snapshot.selected_edges.first().copied());

        let next = match current.and_then(|id| edges.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = edges.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                edges[next_ix]
            }
            None => {
                if forward {
                    edges[0]
                } else {
                    edges[edges.len() - 1]
                }
            }
        };

        self.interaction.focused_edge = Some(next);
        self.interaction.focused_node = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_nodes.clear();
            s.selected_groups.clear();
            s.selected_edges.clear();
            s.selected_edges.push(next);
        });
        true
    }

    pub(super) fn focus_next_node<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let ordered: Vec<GraphNodeId> = self
            .graph
            .read_ref(host, |g| {
                let mut out: Vec<GraphNodeId> = Vec::new();
                let mut used: HashSet<GraphNodeId> = HashSet::new();

                for id in &snapshot.draw_order {
                    if Self::node_is_selectable(g, &snapshot.interaction, *id) && used.insert(*id) {
                        out.push(*id);
                    }
                }

                let mut rest: Vec<GraphNodeId> = g
                    .nodes
                    .keys()
                    .copied()
                    .filter(|id| Self::node_is_selectable(g, &snapshot.interaction, *id))
                    .filter(|id| used.insert(*id))
                    .collect();
                rest.sort_unstable();
                out.extend(rest);
                out
            })
            .ok()
            .unwrap_or_default();

        if ordered.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied());

        let next = match current.and_then(|id| ordered.iter().position(|e| *e == id)) {
            Some(ix) => {
                let len = ordered.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ordered[next_ix]
            }
            None => {
                if forward {
                    ordered[0]
                } else {
                    ordered[ordered.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(next);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = None;
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(next);
            s.draw_order.retain(|id| *id != next);
            s.draw_order.push(next);
        });

        let snapshot = self.sync_view_state(host);
        if snapshot.interaction.auto_pan.on_node_focus {
            self.stop_viewport_animation_timer(host);
            let (geom, _index) = self.canvas_derived(&*host, &snapshot);
            if let Some(ng) = geom.nodes.get(&next) {
                let rect = ng.rect;
                let center = CanvasPoint {
                    x: rect.origin.x.0 + 0.5 * rect.size.width.0,
                    y: rect.origin.y.0 + 0.5 * rect.size.height.0,
                };
                self.ensure_canvas_point_visible(host, &snapshot, center);
            }
        }
        true
    }

    pub(super) fn refresh_focused_port_hints<H: UiHost>(&mut self, host: &mut H) {
        self.interaction.focused_port_valid = false;
        self.interaction.focused_port_convertible = false;

        let snapshot = self.sync_view_state(host);
        let mode = snapshot.interaction.connection_mode;

        let Some(target) = self.interaction.focused_port else {
            return;
        };
        let Some(wire_drag) = self.interaction.wire_drag.clone() else {
            return;
        };

        let presenter = &mut *self.presenter;
        let (valid, convertible) = self
            .graph
            .read_ref(host, |graph| {
                let mut scratch = graph.clone();

                let valid = match &wire_drag.kind {
                    WireDragKind::New { from, bundle } => {
                        let sources = if bundle.is_empty() {
                            std::slice::from_ref(from)
                        } else {
                            bundle.as_slice()
                        };
                        let mut any_accept = false;
                        for src in sources {
                            let plan = presenter.plan_connect(&scratch, *src, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                    WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                        presenter
                            .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode)
                            .decision,
                        ConnectDecision::Accept
                    ),
                    WireDragKind::ReconnectMany { edges } => {
                        let mut any_accept = false;
                        for (edge, endpoint, _fixed) in edges {
                            let plan = presenter
                                .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode);
                            if plan.decision != ConnectDecision::Accept {
                                continue;
                            }
                            any_accept = true;
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                        }
                        any_accept
                    }
                };

                let convertible = if !valid {
                    match &wire_drag.kind {
                        WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                            conversion::is_convertible(presenter, &scratch, *from, target)
                        }
                        _ => false,
                    }
                } else {
                    false
                };

                (valid, convertible)
            })
            .ok()
            .unwrap_or((false, false));

        if self.interaction.wire_drag.is_some() && self.interaction.focused_port == Some(target) {
            self.interaction.focused_port_valid = valid;
            self.interaction.focused_port_convertible = convertible;
        }
    }

    pub(super) fn focus_next_port<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        let snapshot = self.sync_view_state(host);
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let focused_node = self
            .interaction
            .focused_node
            .or_else(|| snapshot.selected_nodes.first().copied())
            .or_else(|| {
                self.graph
                    .read_ref(host, |g| g.nodes.keys().next().copied())
                    .ok()
                    .flatten()
            });

        let Some(focused_node) = focused_node else {
            return false;
        };

        let wire_dir = self.interaction.wire_drag.as_ref().and_then(|w| {
            let from_port = match &w.kind {
                WireDragKind::New { from, .. } => Some(*from),
                WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
                WireDragKind::ReconnectMany { edges } => edges.first().map(|e| e.2),
            }?;
            self.graph
                .read_ref(host, |g| g.ports.get(&from_port).map(|p| p.dir))
                .ok()
                .flatten()
        });

        let ports = self
            .graph
            .read_ref(host, |g| {
                let (inputs, outputs) = node_ports(g, focused_node);
                let mut ports = Vec::with_capacity(inputs.len() + outputs.len());
                ports.extend(inputs);
                ports.extend(outputs);

                if let Some(wire_dir) = wire_dir {
                    let want = match wire_dir {
                        PortDirection::In => PortDirection::Out,
                        PortDirection::Out => PortDirection::In,
                    };
                    ports.retain(|id| g.ports.get(id).is_some_and(|p| p.dir == want));
                }

                ports
            })
            .ok()
            .unwrap_or_default();

        if ports.is_empty() {
            return false;
        }

        let current = self
            .interaction
            .focused_port
            .filter(|id| ports.iter().any(|p| *p == *id));

        let next = match current.and_then(|id| ports.iter().position(|p| *p == id)) {
            Some(ix) => {
                let len = ports.len();
                let next_ix = if forward {
                    (ix + 1) % len
                } else {
                    (ix + len - 1) % len
                };
                ports[next_ix]
            }
            None => {
                if forward {
                    ports[0]
                } else {
                    ports[ports.len() - 1]
                }
            }
        };

        self.interaction.focused_node = Some(focused_node);
        self.interaction.focused_edge = None;
        self.interaction.focused_port = Some(next);
        self.refresh_focused_port_hints(host);
        self.update_view_state(host, |s| {
            s.selected_edges.clear();
            s.selected_groups.clear();
            s.selected_nodes.clear();
            s.selected_nodes.push(focused_node);
        });
        true
    }

    pub(super) fn port_center_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        port: PortId,
    ) -> Option<CanvasPoint> {
        let (geom, _) = self.canvas_derived(&*host, snapshot);
        geom.ports.get(&port).map(|h| CanvasPoint {
            x: h.center.x.0,
            y: h.center.y.0,
        })
    }

    pub(super) fn activate_focused_port<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        if !snapshot.interaction.elements_selectable {
            return false;
        }

        let Some(port) = self
            .interaction
            .focused_port
            .or(self.interaction.hover_port)
        else {
            return false;
        };

        let pos = self
            .port_center_canvas(cx.app, snapshot, port)
            .map(|p| Point::new(Px(p.x), Px(p.y)))
            .or(self.interaction.last_pos)
            .unwrap_or_else(|| {
                let bounds = self.interaction.last_bounds.unwrap_or_default();
                Point::new(
                    Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
                    Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
                )
            });

        if self.interaction.wire_drag.is_none() {
            self.interaction.wire_drag = Some(WireDrag {
                kind: WireDragKind::New {
                    from: port,
                    bundle: Vec::new(),
                },
                pos,
            });
            self.interaction.click_connect = true;
            self.interaction.pending_wire_drag = None;
            self.interaction.suspended_wire_drag = None;
            self.interaction.sticky_wire = false;
            self.interaction.sticky_wire_ignore_next_up = false;
            self.interaction.focused_edge = None;
            self.interaction.focused_port = None;
            self.interaction.focused_port_valid = false;
            self.interaction.focused_port_convertible = false;
            self.interaction.hover_port = None;
            self.interaction.hover_port_valid = false;
            self.interaction.hover_port_convertible = false;
            return true;
        }

        if let Some(mut w) = self.interaction.wire_drag.take() {
            w.pos = pos;
            self.interaction.wire_drag = Some(w);
        }

        let _ = wire_drag::handle_wire_left_up_with_forced_target(
            self,
            cx,
            snapshot,
            snapshot.zoom,
            Some(port),
        );
        self.refresh_focused_port_hints(cx.app);
        true
    }
}
