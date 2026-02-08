use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_open_insert_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_create_group<H: UiHost>(&mut self, cx: &mut CommandCx<'_, H>) -> bool {
        let at = self.interaction.last_canvas_pos.unwrap_or_default();
        self.create_group_at(cx.app, cx.window, at);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }

    pub(super) fn cmd_group_bring_to_front<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_group_send_to_back<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_group_rename<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_open_split_edge_insert_node<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_insert_reroute<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
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

    pub(super) fn cmd_open_conversion_picker<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let Some(ctx0) = self.interaction.last_conversion.clone() else {
            self.show_toast(
                cx.app,
                cx.window,
                DiagnosticSeverity::Info,
                "no recent conversion candidates",
            );
            return true;
        };

        let rows = crate::ui::canvas::searcher::build_rows_flat(&ctx0.candidates, "");
        let visible = rows.len().min(SEARCHER_MAX_VISIBLE_ROWS);
        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let origin = self.clamp_searcher_origin(
            Point::new(Px(ctx0.at.x), Px(ctx0.at.y)),
            visible,
            bounds,
            snapshot,
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
}
