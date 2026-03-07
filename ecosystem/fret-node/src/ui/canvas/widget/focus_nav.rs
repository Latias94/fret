use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn focus_next_edge<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        focus_nav_traversal::focus_next_edge(self, host, forward)
    }

    pub(super) fn focus_next_node<H: UiHost>(&mut self, host: &mut H, forward: bool) -> bool {
        focus_nav_traversal::focus_next_node(self, host, forward)
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
        focus_nav_traversal::focus_next_port(self, host, forward)
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
            self.interaction.hover_port_diagnostic = None;
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
