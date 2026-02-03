use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn align_or_distribute_selection<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        snapshot: &ViewSnapshot,
        mode: AlignDistributeMode,
    ) {
        let selected_nodes = snapshot.selected_nodes.clone();
        let selected_groups = snapshot.selected_groups.clone();
        if selected_nodes.is_empty() && selected_groups.is_empty() {
            return;
        }

        let geom = self.canvas_geometry(&*host, snapshot);

        let ops = self
            .graph
            .read_ref(host, |g| {
                let node_origin = snapshot.interaction.node_origin.normalized();
                #[derive(Clone, Copy)]
                enum ElementId {
                    Node(GraphNodeId),
                    Group(crate::core::GroupId),
                }

                #[derive(Clone, Copy)]
                struct Elem {
                    id: ElementId,
                    x: f32,
                    y: f32,
                    w: f32,
                    h: f32,
                }

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

                let mut elems: Vec<Elem> = Vec::new();
                for node_id in &selected_nodes {
                    let Some(node_geom) = geom.nodes.get(node_id) else {
                        continue;
                    };
                    elems.push(Elem {
                        id: ElementId::Node(*node_id),
                        x: node_geom.rect.origin.x.0,
                        y: node_geom.rect.origin.y.0,
                        w: node_geom.rect.size.width.0,
                        h: node_geom.rect.size.height.0,
                    });
                }
                for group_id in &selected_groups {
                    let Some(group) = g.groups.get(group_id) else {
                        continue;
                    };
                    elems.push(Elem {
                        id: ElementId::Group(*group_id),
                        x: group.rect.origin.x,
                        y: group.rect.origin.y,
                        w: group.rect.size.width,
                        h: group.rect.size.height,
                    });
                }

                if elems.len() < 2 {
                    return Vec::new();
                }

                let mut min_x = f32::INFINITY;
                let mut min_y = f32::INFINITY;
                let mut max_x = f32::NEG_INFINITY;
                let mut max_y = f32::NEG_INFINITY;
                for e in &elems {
                    min_x = min_x.min(e.x);
                    min_y = min_y.min(e.y);
                    max_x = max_x.max(e.x + e.w);
                    max_y = max_y.max(e.y + e.h);
                }
                if !min_x.is_finite()
                    || !min_y.is_finite()
                    || !max_x.is_finite()
                    || !max_y.is_finite()
                {
                    return Vec::new();
                }

                let target_left = min_x;
                let target_top = min_y;
                let target_right = max_x;
                let target_bottom = max_y;
                let target_center_x = 0.5 * (min_x + max_x);
                let target_center_y = 0.5 * (min_y + max_y);

                let mut ops: Vec<GraphOp> = Vec::new();

                let mut per_group_delta: std::collections::HashMap<
                    crate::core::GroupId,
                    CanvasPoint,
                > = std::collections::HashMap::new();
                let mut per_node_delta: std::collections::HashMap<GraphNodeId, CanvasPoint> =
                    std::collections::HashMap::new();

                match mode {
                    AlignDistributeMode::AlignLeft => {
                        for e in &elems {
                            let dx = target_left - e.x;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignRight => {
                        for e in &elems {
                            let new_left = target_right - e.w;
                            let dx = new_left - e.x;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignTop => {
                        for e in &elems {
                            let dy = target_top - e.y;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignBottom => {
                        for e in &elems {
                            let new_top = target_bottom - e.h;
                            let dy = new_top - e.y;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignCenterX => {
                        for e in &elems {
                            let cur = e.x + 0.5 * e.w;
                            let dx = target_center_x - cur;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::AlignCenterY => {
                        for e in &elems {
                            let cur = e.y + 0.5 * e.h;
                            let dy = target_center_y - cur;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::DistributeX => {
                        if elems.len() < 3 {
                            return Vec::new();
                        }
                        let mut sorted = elems;
                        sorted.sort_by(|a, b| {
                            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let first = sorted.first().copied().unwrap();
                        let last = sorted.last().copied().unwrap();
                        let c0 = first.x + 0.5 * first.w;
                        let c1 = last.x + 0.5 * last.w;
                        let span = c1 - c0;
                        if !span.is_finite() || span.abs() <= 1.0e-6 {
                            return Vec::new();
                        }
                        let step = span / (sorted.len() as f32 - 1.0);
                        for (ix, e) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                            let desired = c0 + (ix as f32) * step;
                            let cur = e.x + 0.5 * e.w;
                            let dx = desired - cur;
                            if dx.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: dx, y: 0.0 });
                                }
                            }
                        }
                    }
                    AlignDistributeMode::DistributeY => {
                        if elems.len() < 3 {
                            return Vec::new();
                        }
                        let mut sorted = elems;
                        sorted.sort_by(|a, b| {
                            a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        let first = sorted.first().copied().unwrap();
                        let last = sorted.last().copied().unwrap();
                        let c0 = first.y + 0.5 * first.h;
                        let c1 = last.y + 0.5 * last.h;
                        let span = c1 - c0;
                        if !span.is_finite() || span.abs() <= 1.0e-6 {
                            return Vec::new();
                        }
                        let step = span / (sorted.len() as f32 - 1.0);
                        for (ix, e) in sorted.iter().enumerate().skip(1).take(sorted.len() - 2) {
                            let desired = c0 + (ix as f32) * step;
                            let cur = e.y + 0.5 * e.h;
                            let dy = desired - cur;
                            if dy.abs() <= 1.0e-9 {
                                continue;
                            }
                            match e.id {
                                ElementId::Group(id) => {
                                    per_group_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                                ElementId::Node(id) => {
                                    per_node_delta.insert(id, CanvasPoint { x: 0.0, y: dy });
                                }
                            }
                        }
                    }
                }

                let aligns = matches!(
                    mode,
                    AlignDistributeMode::AlignLeft
                        | AlignDistributeMode::AlignRight
                        | AlignDistributeMode::AlignTop
                        | AlignDistributeMode::AlignBottom
                        | AlignDistributeMode::AlignCenterX
                        | AlignDistributeMode::AlignCenterY
                );
                let affects_x = matches!(
                    mode,
                    AlignDistributeMode::AlignLeft
                        | AlignDistributeMode::AlignRight
                        | AlignDistributeMode::AlignCenterX
                );
                let affects_y = matches!(
                    mode,
                    AlignDistributeMode::AlignTop
                        | AlignDistributeMode::AlignBottom
                        | AlignDistributeMode::AlignCenterY
                );

                let moved_nodes: std::collections::HashSet<GraphNodeId> = selected_nodes
                    .iter()
                    .copied()
                    .chain(moved_by_group.iter().copied())
                    .collect();
                let multi_move = moved_nodes.len() > 1;
                let skip_node_extent_clamp = aligns && multi_move;

                let mut shift = CanvasPoint::default();
                let any_delta = !per_group_delta.is_empty() || !per_node_delta.is_empty();
                if aligns
                    && any_delta
                    && multi_move
                    && let Some(extent) = snapshot.interaction.node_extent
                {
                    let mut min_x: f32 = f32::INFINITY;
                    let mut min_y: f32 = f32::INFINITY;
                    let mut max_x: f32 = f32::NEG_INFINITY;
                    let mut max_y: f32 = f32::NEG_INFINITY;
                    let mut any = false;

                    for node_id in moved_nodes.iter().copied() {
                        let Some(node_geom) = geom.nodes.get(&node_id) else {
                            continue;
                        };
                        let w = node_geom.rect.size.width.0.max(0.0);
                        let h = node_geom.rect.size.height.0.max(0.0);
                        if !w.is_finite() || !h.is_finite() {
                            continue;
                        }

                        let base_delta = if moved_by_group.contains(&node_id) {
                            g.nodes
                                .get(&node_id)
                                .and_then(|n| n.parent)
                                .and_then(|p| per_group_delta.get(&p).copied())
                                .unwrap_or_default()
                        } else {
                            per_node_delta.get(&node_id).copied().unwrap_or_default()
                        };

                        let x0 = node_geom.rect.origin.x.0 + base_delta.x;
                        let y0 = node_geom.rect.origin.y.0 + base_delta.y;
                        if !x0.is_finite() || !y0.is_finite() {
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
                        let bbox_w = (max_x - min_x).max(0.0);
                        let bbox_h = (max_y - min_y).max(0.0);
                        let extent_w = extent.size.width.max(0.0);
                        let extent_h = extent.size.height.max(0.0);

                        if affects_x
                            && extent.origin.x.is_finite()
                            && extent_w.is_finite()
                            && bbox_w.is_finite()
                        {
                            let min_dx = extent.origin.x - min_x;
                            let mut max_dx = extent.origin.x + (extent_w - bbox_w).max(0.0) - min_x;
                            if !max_dx.is_finite() || max_dx < min_dx {
                                max_dx = min_dx;
                            }
                            shift.x = 0.0_f32.clamp(min_dx, max_dx);
                        }

                        if affects_y
                            && extent.origin.y.is_finite()
                            && extent_h.is_finite()
                            && bbox_h.is_finite()
                        {
                            let min_dy = extent.origin.y - min_y;
                            let mut max_dy = extent.origin.y + (extent_h - bbox_h).max(0.0) - min_y;
                            if !max_dy.is_finite() || max_dy < min_dy {
                                max_dy = min_dy;
                            }
                            shift.y = 0.0_f32.clamp(min_dy, max_dy);
                        }
                    }
                }

                // Apply group deltas first (and move their child nodes).
                let mut groups_sorted = selected_groups.clone();
                groups_sorted.sort();
                for group_id in groups_sorted {
                    let base = per_group_delta.get(&group_id).copied().unwrap_or_default();
                    let mut delta = CanvasPoint {
                        x: base.x + shift.x,
                        y: base.y + shift.y,
                    };

                    if delta.x.abs() > 1.0e-9 || delta.y.abs() > 1.0e-9 {
                        let mut min_dx: f32 = f32::NEG_INFINITY;
                        let mut max_dx: f32 = f32::INFINITY;
                        let mut min_dy: f32 = f32::NEG_INFINITY;
                        let mut max_dy: f32 = f32::INFINITY;
                        let mut any_x = false;
                        let mut any_y = false;

                        for (&node_id, node) in &g.nodes {
                            if node.parent != Some(group_id) {
                                continue;
                            }
                            let Some(crate::core::NodeExtent::Rect { rect }) = node.extent else {
                                continue;
                            };

                            let node_size = if let Some(node_geom) = geom.nodes.get(&node_id) {
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

                    if !aligns
                        && (delta.x.abs() > 1.0e-9 || delta.y.abs() > 1.0e-9)
                        && let Some(extent) = snapshot.interaction.node_extent
                    {
                        let mut min_x: f32 = f32::INFINITY;
                        let mut min_y: f32 = f32::INFINITY;
                        let mut max_x: f32 = f32::NEG_INFINITY;
                        let mut max_y: f32 = f32::NEG_INFINITY;
                        let mut any = false;

                        for (&node_id, node) in &g.nodes {
                            if node.parent != Some(group_id) {
                                continue;
                            }

                            let (x0, y0, w, h) = if let Some(node_geom) = geom.nodes.get(&node_id) {
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
                            if !x0.is_finite()
                                || !y0.is_finite()
                                || !w.is_finite()
                                || !h.is_finite()
                            {
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
                            let bbox_w = (max_x - min_x).max(0.0);
                            let bbox_h = (max_y - min_y).max(0.0);
                            let extent_w = extent.size.width.max(0.0);
                            let extent_h = extent.size.height.max(0.0);

                            if min_x.is_finite()
                                && bbox_w.is_finite()
                                && extent.origin.x.is_finite()
                                && extent_w.is_finite()
                            {
                                let min_dx = extent.origin.x - min_x;
                                let mut max_dx =
                                    extent.origin.x + (extent_w - bbox_w).max(0.0) - min_x;
                                if !max_dx.is_finite() || max_dx < min_dx {
                                    max_dx = min_dx;
                                }
                                delta.x = delta.x.clamp(min_dx, max_dx);
                            }

                            if min_y.is_finite()
                                && bbox_h.is_finite()
                                && extent.origin.y.is_finite()
                                && extent_h.is_finite()
                            {
                                let min_dy = extent.origin.y - min_y;
                                let mut max_dy =
                                    extent.origin.y + (extent_h - bbox_h).max(0.0) - min_y;
                                if !max_dy.is_finite() || max_dy < min_dy {
                                    max_dy = min_dy;
                                }
                                delta.y = delta.y.clamp(min_dy, max_dy);
                            }
                        }
                    }
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

                    if delta.x.abs() <= 1.0e-9 && delta.y.abs() <= 1.0e-9 {
                        continue;
                    }
                    for (&node_id, node) in &g.nodes {
                        if node.parent != Some(group_id) {
                            continue;
                        }
                        let from = node.pos;
                        let to = CanvasPoint {
                            x: from.x + delta.x,
                            y: from.y + delta.y,
                        };
                        if from != to {
                            ops.push(GraphOp::SetNodePos {
                                id: node_id,
                                from,
                                to,
                            });
                        }
                    }
                }

                // Apply node deltas for nodes not moved by a selected group.
                let mut nodes_sorted = selected_nodes.clone();
                nodes_sorted.sort();
                for node_id in nodes_sorted {
                    if moved_by_group.contains(&node_id) {
                        continue;
                    }
                    let base = per_node_delta.get(&node_id).copied().unwrap_or_default();
                    let delta = CanvasPoint {
                        x: base.x + shift.x,
                        y: base.y + shift.y,
                    };
                    let moved = delta.x.abs() > 1.0e-9 || delta.y.abs() > 1.0e-9;
                    let Some(node) = g.nodes.get(&node_id) else {
                        continue;
                    };
                    let from = node.pos;
                    let mut to = CanvasPoint {
                        x: from.x + delta.x,
                        y: from.y + delta.y,
                    };

                    // Reuse the same extent constraints as drag/nudge.
                    let node_size = if let Some(node_geom) = geom.nodes.get(&node_id) {
                        Some(CanvasSize {
                            width: node_geom.rect.size.width.0,
                            height: node_geom.rect.size.height.0,
                        })
                    } else {
                        node.size
                    };

                    if let Some(node_size) = node_size {
                        let node_w = node_size.width;
                        let node_h = node_size.height;

                        if moved
                            && !skip_node_extent_clamp
                            && let Some(extent) = snapshot.interaction.node_extent
                        {
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

                        if moved && let Some(crate::core::NodeExtent::Rect { rect }) = node.extent {
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

        let label = match mode {
            AlignDistributeMode::AlignLeft => "Align Left",
            AlignDistributeMode::AlignRight => "Align Right",
            AlignDistributeMode::AlignTop => "Align Top",
            AlignDistributeMode::AlignBottom => "Align Bottom",
            AlignDistributeMode::AlignCenterX => "Align Center X",
            AlignDistributeMode::AlignCenterY => "Align Center Y",
            AlignDistributeMode::DistributeX => "Distribute X",
            AlignDistributeMode::DistributeY => "Distribute Y",
        };
        let _ = self.commit_ops(host, window, Some(label), ops);
    }
}
