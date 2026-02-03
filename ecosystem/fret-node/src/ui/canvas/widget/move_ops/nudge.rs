use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn nudge_selection_by_screen_delta<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        delta_screen_px: CanvasPoint,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let zoom = snapshot.zoom;
        if !zoom.is_finite() || zoom <= 0.0 {
            return;
        }

        let mut delta = CanvasPoint {
            x: fret_canvas::scale::canvas_units_from_screen_px(delta_screen_px.x, zoom),
            y: fret_canvas::scale::canvas_units_from_screen_px(delta_screen_px.y, zoom),
        };
        if !delta.x.is_finite() || !delta.y.is_finite() {
            return;
        }

        if snapshot.interaction.snap_to_grid {
            if let Some(primary) = selected_nodes.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.nodes.get(&primary).map(|n| n.pos))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            } else if let Some(primary) = selected_groups.first().copied() {
                let primary_start = self
                    .graph
                    .read_ref(host, |g| g.groups.get(&primary).map(|gr| gr.rect.origin))
                    .ok()
                    .flatten()
                    .unwrap_or_default();
                let primary_target = CanvasPoint {
                    x: primary_start.x + delta.x,
                    y: primary_start.y + delta.y,
                };
                let snapped =
                    Self::snap_canvas_point(primary_target, snapshot.interaction.snap_grid);
                delta = CanvasPoint {
                    x: snapped.x - primary_start.x,
                    y: snapped.y - primary_start.y,
                };
            }
        }

        if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
            return;
        }

        let geom_for_extent = self.canvas_geometry(&*host, snapshot);
        let ops = self
            .graph
            .read_ref(host, |g| {
                let mut delta = delta;
                let mut ops: Vec<GraphOp> = Vec::new();
                let node_origin = snapshot.interaction.node_origin.normalized();

                let selected_groups_set: std::collections::HashSet<crate::core::GroupId> =
                    selected_groups.iter().copied().collect();

                let mut moved_by_group: std::collections::HashSet<GraphNodeId> =
                    std::collections::HashSet::new();
                for (&node_id, node) in &g.nodes {
                    if let Some(parent) = node.parent
                        && selected_groups_set.contains(&parent)
                    {
                        moved_by_group.insert(node_id);
                    }
                }

                let mut moved_nodes: std::collections::BTreeSet<GraphNodeId> =
                    selected_nodes.iter().copied().collect();
                for id in &moved_by_group {
                    moved_nodes.insert(*id);
                }

                let shared_move = !selected_groups.is_empty() || moved_nodes.len() > 1;

                if shared_move && let Some(extent) = snapshot.interaction.node_extent {
                    let mut min_x: f32 = f32::INFINITY;
                    let mut min_y: f32 = f32::INFINITY;
                    let mut max_x: f32 = f32::NEG_INFINITY;
                    let mut max_y: f32 = f32::NEG_INFINITY;
                    let mut any = false;

                    for node_id in moved_nodes.iter().copied() {
                        let Some(node) = g.nodes.get(&node_id) else {
                            continue;
                        };

                        let (x0, y0, w, h) =
                            if let Some(node_geom) = geom_for_extent.nodes.get(&node_id) {
                                (
                                    node_geom.rect.origin.x.0,
                                    node_geom.rect.origin.y.0,
                                    node_geom.rect.size.width.0.max(0.0),
                                    node_geom.rect.size.height.0.max(0.0),
                                )
                            } else if let Some(size) = node.size {
                                (
                                    node.pos.x,
                                    node.pos.y,
                                    size.width.max(0.0),
                                    size.height.max(0.0),
                                )
                            } else {
                                continue;
                            };
                        if !x0.is_finite() || !y0.is_finite() || !w.is_finite() || !h.is_finite() {
                            continue;
                        }

                        any = true;
                        min_x = min_x.min(x0);
                        min_y = min_y.min(y0);
                        max_x = max_x.max(x0 + w);
                        max_y = max_y.max(y0 + h);
                    }

                    if any
                        && min_x.is_finite()
                        && min_y.is_finite()
                        && max_x.is_finite()
                        && max_y.is_finite()
                    {
                        let group_min = CanvasPoint { x: min_x, y: min_y };
                        let group_size = CanvasSize {
                            width: (max_x - min_x).max(0.0),
                            height: (max_y - min_y).max(0.0),
                        };
                        let group_w = group_size.width.max(0.0);
                        let group_h = group_size.height.max(0.0);
                        let extent_w = extent.size.width.max(0.0);
                        let extent_h = extent.size.height.max(0.0);

                        let min_dx = extent.origin.x - group_min.x;
                        let mut max_dx = extent.origin.x + (extent_w - group_w) - group_min.x;
                        if !max_dx.is_finite() || max_dx < min_dx {
                            max_dx = min_dx;
                        }
                        delta.x = delta.x.clamp(min_dx, max_dx);

                        let min_dy = extent.origin.y - group_min.y;
                        let mut max_dy = extent.origin.y + (extent_h - group_h) - group_min.y;
                        if !max_dy.is_finite() || max_dy < min_dy {
                            max_dy = min_dy;
                        }
                        delta.y = delta.y.clamp(min_dy, max_dy);
                    }
                }

                if shared_move {
                    let mut min_dx: f32 = f32::NEG_INFINITY;
                    let mut max_dx: f32 = f32::INFINITY;
                    let mut min_dy: f32 = f32::NEG_INFINITY;
                    let mut max_dy: f32 = f32::INFINITY;
                    let mut any_x = false;
                    let mut any_y = false;

                    for node_id in moved_nodes.iter().copied() {
                        let Some(node) = g.nodes.get(&node_id) else {
                            continue;
                        };
                        let Some(crate::core::NodeExtent::Rect { rect }) = node.extent else {
                            continue;
                        };

                        let node_size = if let Some(node_geom) = geom_for_extent.nodes.get(&node_id)
                        {
                            Some(CanvasSize {
                                width: node_geom.rect.size.width.0,
                                height: node_geom.rect.size.height.0,
                            })
                        } else {
                            node.size
                        };
                        let Some(node_size) = node_size else {
                            continue;
                        };
                        let node_w = node_size.width.max(0.0);
                        let node_h = node_size.height.max(0.0);
                        if !node_w.is_finite() || !node_h.is_finite() {
                            continue;
                        }

                        let min_x = rect.origin.x;
                        let max_x = rect.origin.x + (rect.size.width - node_w).max(0.0);
                        if min_x.is_finite() && max_x.is_finite() && node.pos.x.is_finite() {
                            any_x = true;
                            min_dx = min_dx.max(min_x - node.pos.x);
                            max_dx = max_dx.min(max_x - node.pos.x);
                        }

                        let min_y = rect.origin.y;
                        let max_y = rect.origin.y + (rect.size.height - node_h).max(0.0);
                        if min_y.is_finite() && max_y.is_finite() && node.pos.y.is_finite() {
                            any_y = true;
                            min_dy = min_dy.max(min_y - node.pos.y);
                            max_dy = max_dy.min(max_y - node.pos.y);
                        }
                    }

                    if any_x && min_dx.is_finite() && max_dx.is_finite() {
                        if max_dx < min_dx {
                            max_dx = min_dx;
                        }
                        delta.x = delta.x.clamp(min_dx, max_dx);
                    }
                    if any_y && min_dy.is_finite() && max_dy.is_finite() {
                        if max_dy < min_dy {
                            max_dy = min_dy;
                        }
                        delta.y = delta.y.clamp(min_dy, max_dy);
                    }
                }

                let mut groups_sorted = selected_groups.clone();
                groups_sorted.sort();
                for group_id in groups_sorted {
                    let Some(group) = g.groups.get(&group_id) else {
                        continue;
                    };
                    let from = group.rect;
                    let to = crate::core::CanvasRect {
                        origin: CanvasPoint {
                            x: from.origin.x + delta.x,
                            y: from.origin.y + delta.y,
                        },
                        size: from.size,
                    };
                    if from != to {
                        ops.push(GraphOp::SetGroupRect {
                            id: group_id,
                            from,
                            to,
                        });
                    }
                }

                for node_id in moved_nodes {
                    let Some(node) = g.nodes.get(&node_id) else {
                        continue;
                    };
                    let from = node.pos;
                    let mut to = CanvasPoint {
                        x: from.x + delta.x,
                        y: from.y + delta.y,
                    };

                    if !moved_by_group.contains(&node_id) {
                        let node_size = if let Some(node_geom) = geom_for_extent.nodes.get(&node_id)
                        {
                            Some(CanvasSize {
                                width: node_geom.rect.size.width.0,
                                height: node_geom.rect.size.height.0,
                            })
                        } else {
                            node.size
                        };
                        let Some(node_size) = node_size else {
                            continue;
                        };
                        let node_w = node_size.width;
                        let node_h = node_size.height;

                        if !shared_move && let Some(extent) = snapshot.interaction.node_extent {
                            let min_x = extent.origin.x;
                            let min_y = extent.origin.y;
                            let max_x = extent.origin.x + (extent.size.width - node_w).max(0.0);
                            let max_y = extent.origin.y + (extent.size.height - node_h).max(0.0);
                            let mut rect_origin =
                                node_rect_origin_from_anchor(to, node_size, node_origin);
                            rect_origin.x = rect_origin.x.clamp(min_x, max_x);
                            rect_origin.y = rect_origin.y.clamp(min_y, max_y);
                            to = node_anchor_from_rect_origin(rect_origin, node_size, node_origin);
                        }

                        if let Some(crate::core::NodeExtent::Rect { rect }) = node.extent {
                            let min_x = rect.origin.x;
                            let min_y = rect.origin.y;
                            let max_x = rect.origin.x + (rect.size.width - node_w).max(0.0);
                            let max_y = rect.origin.y + (rect.size.height - node_h).max(0.0);
                            let mut rect_origin =
                                node_rect_origin_from_anchor(to, node_size, node_origin);
                            rect_origin.x = rect_origin.x.clamp(min_x, max_x);
                            rect_origin.y = rect_origin.y.clamp(min_y, max_y);
                            to = node_anchor_from_rect_origin(rect_origin, node_size, node_origin);
                        }

                        if let Some(parent) = node.parent
                            && let Some(group) = g.groups.get(&parent)
                        {
                            let min_x = group.rect.origin.x;
                            let min_y = group.rect.origin.y;
                            let max_x =
                                group.rect.origin.x + (group.rect.size.width - node_w).max(0.0);
                            let max_y =
                                group.rect.origin.y + (group.rect.size.height - node_h).max(0.0);
                            let mut rect_origin =
                                node_rect_origin_from_anchor(to, node_size, node_origin);
                            rect_origin.x = rect_origin.x.clamp(min_x, max_x);
                            rect_origin.y = rect_origin.y.clamp(min_y, max_y);
                            to = node_anchor_from_rect_origin(rect_origin, node_size, node_origin);
                        }
                    }

                    if from != to {
                        ops.push(GraphOp::SetNodePos {
                            id: node_id,
                            from,
                            to,
                        });
                    }
                }

                ops
            })
            .ok()
            .unwrap_or_default();

        if ops.is_empty() {
            return;
        }

        let _ = self.commit_ops(host, window, Some("Nudge"), ops);
    }
}
