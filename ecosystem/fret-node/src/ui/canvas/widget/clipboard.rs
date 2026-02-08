use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn next_paste_canvas_point(
        &mut self,
        bounds: Rect,
        snapshot: &ViewSnapshot,
    ) -> CanvasPoint {
        let zoom = if snapshot.zoom.is_finite() && snapshot.zoom > 0.0 {
            snapshot.zoom
        } else {
            1.0
        };

        let anchor = self.interaction.last_canvas_pos.unwrap_or_else(|| {
            let cx0 = bounds.origin.x.0 + 0.5 * bounds.size.width.0;
            let cy0 = bounds.origin.y.0 + 0.5 * bounds.size.height.0;
            let center = Point::new(Px(cx0), Px(cy0));
            Self::screen_to_canvas(bounds, center, snapshot.pan, zoom)
        });

        let (series, at) = PasteSeries::next(self.interaction.paste_series, anchor, zoom);
        self.interaction.paste_series = Some(series);
        at
    }

    pub(super) fn copy_selection_to_clipboard<H: UiHost>(
        &mut self,
        host: &mut H,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }
        let text = self
            .graph
            .read_ref(host, |graph| {
                let fragment = GraphFragment::from_selection(
                    graph,
                    selected_nodes.to_vec(),
                    selected_groups.to_vec(),
                );
                fragment.to_clipboard_text().unwrap_or_default()
            })
            .ok()
            .unwrap_or_default();
        if text.is_empty() {
            return;
        }
        host.push_effect(Effect::ClipboardSetText { text });
    }

    pub(super) fn request_paste_at_canvas<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        at: CanvasPoint,
    ) {
        let Some(window) = window else {
            return;
        };

        let token = host.next_clipboard_token();
        self.interaction.pending_paste = Some(PendingPaste { token, at });
        host.push_effect(Effect::ClipboardGetText { window, token });
    }

    pub(super) fn apply_paste_text<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        text: &str,
        at: CanvasPoint,
    ) {
        let fragment: GraphFragment = match GraphFragment::from_clipboard_text(text) {
            Ok(v) => v,
            Err(_) => {
                self.show_toast(
                    host,
                    window,
                    DiagnosticSeverity::Info,
                    "clipboard does not contain a fret-node fragment",
                );
                return;
            }
        };

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        for node in fragment.nodes.values() {
            min_x = min_x.min(node.pos.x);
            min_y = min_y.min(node.pos.y);
        }
        for group in fragment.groups.values() {
            min_x = min_x.min(group.rect.origin.x);
            min_y = min_y.min(group.rect.origin.y);
        }
        if !min_x.is_finite() || !min_y.is_finite() {
            return;
        }

        let tuning = PasteTuning {
            offset: CanvasPoint {
                x: at.x - min_x,
                y: at.y - min_y,
            },
        };
        let remapper = IdRemapper::new(IdRemapSeed::new_random());
        let mut tx = fragment.to_paste_transaction(&remapper, tuning);
        if !fragment.imports.is_empty() {
            self.graph
                .read_ref(host, |graph| {
                    tx.ops.retain(|op| {
                        !matches!(op, GraphOp::AddImport { id, .. } if graph.imports.contains_key(id))
                    });
                })
                .ok();
        }

        let new_nodes: Vec<GraphNodeId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        let new_groups: Vec<crate::core::GroupId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect();

        if !self.apply_ops_result(host, window, tx.ops) {
            return;
        }

        if !new_nodes.is_empty() || !new_groups.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                s.selected_groups = new_groups.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
                for id in &new_groups {
                    s.group_draw_order.retain(|x| x != id);
                    s.group_draw_order.push(*id);
                }
            });
        }
    }

    pub(super) fn duplicate_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        selected_nodes: &[GraphNodeId],
        selected_groups: &[crate::core::GroupId],
    ) {
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let fragment = self
            .graph
            .read_ref(host, |graph| {
                GraphFragment::from_selection(
                    graph,
                    selected_nodes.to_vec(),
                    selected_groups.to_vec(),
                )
            })
            .ok()
            .unwrap_or_default();

        let tuning = PasteTuning {
            offset: CanvasPoint { x: 24.0, y: 24.0 },
        };
        let remapper = IdRemapper::new(IdRemapSeed::new_random());
        let mut tx = fragment.to_paste_transaction(&remapper, tuning);
        if !fragment.imports.is_empty() {
            self.graph
                .read_ref(host, |graph| {
                    tx.ops.retain(|op| {
                        !matches!(op, GraphOp::AddImport { id, .. } if graph.imports.contains_key(id))
                    });
                })
                .ok();
        }
        tx.label = Some("Duplicate".to_string());

        let new_nodes: Vec<GraphNodeId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        let new_groups: Vec<crate::core::GroupId> = tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect();

        if !self.commit_transaction(host, window, &tx) {
            return;
        }

        if !new_nodes.is_empty() || !new_groups.is_empty() {
            self.update_view_state(host, |s| {
                s.selected_edges.clear();
                s.selected_nodes = new_nodes.clone();
                s.selected_groups = new_groups.clone();
                for id in &new_nodes {
                    s.draw_order.retain(|x| x != id);
                    s.draw_order.push(*id);
                }
                for id in &new_groups {
                    s.group_draw_order.retain(|x| x != id);
                    s.group_draw_order.push(*id);
                }
            });
        }
    }
}
