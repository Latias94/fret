use super::command_ui::finish_command_paint;
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
        finish_command_paint(cx)
    }

    pub(super) fn cmd_create_group<H: UiHost>(&mut self, cx: &mut CommandCx<'_, H>) -> bool {
        let at = self.interaction.last_canvas_pos.unwrap_or_default();
        self.create_group_at(cx.app, cx.window, at);
        finish_command_paint(cx)
    }

    pub(super) fn cmd_group_bring_to_front<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.dismiss_command_transients();
        let groups = snapshot.selected_groups.clone();
        if groups.is_empty() {
            return true;
        }
        self.update_view_state(cx.app, |s| {
            group_draw_order::bring_selected_groups_to_front_in_view_state(s, &groups);
        });
        finish_command_paint(cx)
    }

    pub(super) fn cmd_group_send_to_back<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.dismiss_command_transients();
        let groups = snapshot.selected_groups.clone();
        if groups.is_empty() {
            return true;
        }
        self.update_view_state(cx.app, |s| {
            group_draw_order::send_selected_groups_to_back_in_view_state(s, &groups);
        });
        finish_command_paint(cx)
    }

    pub(super) fn cmd_group_rename<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        self.dismiss_command_transients();
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
        let invoked_at = self.command_invoked_at();
        let _ = overlays.update(cx.app, |s, _cx| {
            s.group_rename = Some(GroupRenameOverlay {
                group,
                invoked_at_window: invoked_at,
            });
        });
        finish_command_paint(cx)
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
        let invoked_at = self.command_invoked_at();
        self.open_edge_insert_node_picker(cx.app, cx.window, edge, invoked_at);
        finish_command_paint(cx)
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
        let invoked_at = self.command_invoked_at();
        let outcome = self.plan_canvas_split_edge_reroute(cx.app, edge_id, invoked_at);
        self.execute_split_edge_reroute_outcome(cx.app, cx.window, Some("Insert Reroute"), outcome);

        finish_command_paint(cx)
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

        let bounds = self.interaction.last_bounds.unwrap_or_default();
        let invoked_at = Point::new(Px(ctx0.at.x), Px(ctx0.at.y));

        self.dismiss_command_context_menu();
        self.open_searcher_overlay(
            invoked_at,
            bounds,
            snapshot,
            ContextMenuTarget::ConnectionConvertPicker {
                from: ctx0.from,
                to: ctx0.to,
                at: ctx0.at,
            },
            ctx0.candidates,
            SearcherRowsMode::Flat,
        );

        finish_command_paint(cx)
    }
}
