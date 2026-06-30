use super::*;

use std::cmp::Ordering;

const MAX_CHILDREN: usize = 12;

fn hit_test_bounds_tree_disabled() -> bool {
    crate::runtime_config::ui_runtime_config().hit_test_bounds_tree_disabled
}

fn hit_test_bounds_tree_min_records() -> usize {
    crate::runtime_config::ui_runtime_config().hit_test_bounds_tree_min_records
}

#[derive(Debug, Default)]
pub(super) struct HitTestBoundsTrees {
    frame_id: Option<FrameId>,
    clip_stack: Vec<(NodeId, Option<Rect>, Transform2D)>,
    leaves: Vec<Leaf>,
    search_stack: Vec<usize>,
    level: Vec<usize>,
    next_level: Vec<usize>,
}

impl HitTestBoundsTrees {
    pub(super) fn clear(&mut self) {
        self.frame_id = None;
        self.clip_stack.clear();
        self.leaves.clear();
        self.search_stack.clear();
        self.level.clear();
        self.next_level.clear();
    }

    pub(super) fn begin_frame(&mut self, frame_id: FrameId) -> bool {
        if hit_test_bounds_tree_disabled() {
            self.clear();
            return false;
        }

        self.frame_id = Some(frame_id);
        true
    }

    fn frame_active(&self) -> bool {
        self.frame_id.is_some() && !hit_test_bounds_tree_disabled()
    }

    pub(super) fn rebuild_for_layer_from_records<H: UiHost>(
        &mut self,
        layer_root: NodeId,
        records: &[prepaint::InteractionRecord],
        nodes: &SlotMap<NodeId, Node<H>>,
        layer: &mut BoundaryHitTestBoundsState,
    ) {
        let Some(frame_id) = self.frame_id else {
            layer.clear();
            return;
        };

        if records.len() < hit_test_bounds_tree_min_records() {
            layer.disable_for_frame(frame_id);
            return;
        }
        if records.first().is_none_or(|r| r.node != layer_root) {
            layer.disable_for_frame(frame_id);
            return;
        }

        self.clip_stack.clear();
        self.leaves.clear();

        let mut disabled = false;
        let mut order: u32 = 1;

        for record in records {
            let parent = nodes.get(record.node).and_then(|n| n.parent);
            if record.node == layer_root {
                self.clip_stack.clear();
            } else {
                while self
                    .clip_stack
                    .last()
                    .is_some_and(|(ancestor, _, _)| Some(*ancestor) != parent)
                {
                    self.clip_stack.pop();
                }
                if parent.is_some() && self.clip_stack.is_empty() {
                    disabled = true;
                    break;
                }
            }

            let (parent_clip_world, parent_to_world_for_children) = self
                .clip_stack
                .last()
                .map(|(_, clip, t)| (*clip, *t))
                .unwrap_or((None, Transform2D::IDENTITY));

            let render_to_world = record
                .render_transform_inv
                .and_then(|inv| inv.inverse())
                .unwrap_or(Transform2D::IDENTITY);
            let children_to_world = record
                .children_render_transform_inv
                .and_then(|inv| inv.inverse())
                .unwrap_or(Transform2D::IDENTITY);
            if !transform_is_axis_aligned(render_to_world)
                || !transform_is_axis_aligned(children_to_world)
            {
                disabled = true;
                break;
            }

            let node_to_world = parent_to_world_for_children * render_to_world;
            let node_bounds_world = rect_transform_aabb(record.bounds, node_to_world);
            let effective_world = parent_clip_world.map_or(node_bounds_world, |clip| {
                rect_intersection(clip, node_bounds_world)
            });

            let clip_world_for_children = if record.clips_hit_test {
                Some(rect_intersection(effective_world, node_bounds_world))
            } else {
                parent_clip_world
            };
            let to_world_for_children = node_to_world * children_to_world;

            // Maintain the stack even if the clip is empty so children inherit the empty clip.
            self.clip_stack
                .push((record.node, clip_world_for_children, to_world_for_children));

            if rect_is_empty(effective_world) {
                continue;
            }

            self.leaves.push(Leaf {
                bounds: effective_world,
                center_x: rect_center_x(effective_world),
                center_y: rect_center_y(effective_world),
                order,
                node: record.node,
            });
            order = order.saturating_add(1);
        }

        let disabled_or_empty = disabled || self.leaves.is_empty();

        let mut tree = layer.take_tree_for_rebuild(frame_id);

        if disabled_or_empty {
            tree.clear_keep_alloc();
            layer.finish_rebuild(frame_id, false, tree);
            return;
        }

        tree.rebuild_from_leaves(&mut self.leaves, &mut self.level, &mut self.next_level);
        layer.finish_rebuild(frame_id, true, tree);
    }

    pub(super) fn query(
        &mut self,
        layer: Option<&BoundaryHitTestBoundsState>,
        position: Point,
        collect_stats: bool,
    ) -> (HitTestBoundsTreeQuery, HitTestBoundsTreeQueryStats) {
        let Some(frame_id) = self.frame_id else {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        };
        let Some(layer) = layer else {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        };
        if !layer.enabled_for_frame(frame_id) {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        }
        let (hit, stats) =
            layer
                .tree
                .find_max_containing_point(position, collect_stats, &mut self.search_stack);
        let query = match hit {
            Some(hit) => HitTestBoundsTreeQuery::Hit(hit),
            None => HitTestBoundsTreeQuery::Miss,
        };
        (query, stats)
    }

    pub(super) fn reuse_for_layer(&mut self, layer: &mut BoundaryHitTestBoundsState) {
        let Some(frame_id) = self.frame_id else {
            return;
        };

        if layer.can_reuse_for_stable_frame() {
            layer.reuse_for_frame(frame_id);
        }
    }

    pub(super) fn layer_enabled(&self, layer: Option<&BoundaryHitTestBoundsState>) -> bool {
        if hit_test_bounds_tree_disabled() {
            return false;
        }
        let Some(frame_id) = self.frame_id else {
            return false;
        };
        layer.is_some_and(|layer| layer.enabled_for_frame(frame_id))
    }
}

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn clear_hit_test_bounds_frame_products(&mut self) {
        self.hit_test_bounds_trees.clear();
        for (_, boundary) in self.view_boundaries.iter_mut() {
            boundary.frame_products.hit_test_bounds.clear();
        }
    }

    pub(in crate::tree) fn begin_hit_test_bounds_frame(&mut self, frame_id: FrameId) {
        if !self.hit_test_bounds_trees.begin_frame(frame_id) {
            self.clear_hit_test_bounds_frame_products();
            return;
        }

        for (_, boundary) in self.view_boundaries.iter_mut() {
            boundary.frame_products.hit_test_bounds.mark_unused();
        }
    }

    pub(in crate::tree) fn rebuild_hit_test_bounds_for_layer_from_interaction_range(
        &mut self,
        layer_root: NodeId,
        start: usize,
        end: usize,
    ) {
        if !self.hit_test_bounds_trees.frame_active() || self.nodes.get(layer_root).is_none() {
            return;
        }

        let has_existing_boundary = self.view_boundaries.contains_key(layer_root);
        let is_rebuild_candidate = {
            let records = &self.interaction_cache.records[start..end];
            records.len() >= hit_test_bounds_tree_min_records()
                && records
                    .first()
                    .is_some_and(|record| record.node == layer_root)
        };
        if !has_existing_boundary && !is_rebuild_candidate {
            return;
        }

        let _ = self.ensure_view_boundary_state(layer_root);
        let records = &self.interaction_cache.records[start..end];
        let nodes = &self.nodes;
        let hit_test_bounds_trees = &mut self.hit_test_bounds_trees;
        let Some(boundary) = self.view_boundaries.get_mut(layer_root) else {
            return;
        };
        hit_test_bounds_trees.rebuild_for_layer_from_records(
            layer_root,
            records,
            nodes,
            &mut boundary.frame_products.hit_test_bounds,
        );
    }

    pub(in crate::tree) fn reuse_hit_test_bounds_for_layer(&mut self, layer_root: NodeId) {
        if !self.hit_test_bounds_trees.frame_active() || self.nodes.get(layer_root).is_none() {
            return;
        }

        let Some(boundary) = self.view_boundaries.get_mut(layer_root) else {
            return;
        };
        let hit_test_bounds_trees = &mut self.hit_test_bounds_trees;
        hit_test_bounds_trees.reuse_for_layer(&mut boundary.frame_products.hit_test_bounds);
    }

    pub(in crate::tree) fn hit_test_bounds_tree_layer_enabled(&self, layer_root: NodeId) -> bool {
        let layer = self
            .view_boundaries
            .get(layer_root)
            .map(|boundary| &boundary.frame_products.hit_test_bounds);
        self.hit_test_bounds_trees.layer_enabled(layer)
    }

    pub(in crate::tree) fn query_hit_test_bounds_tree(
        &mut self,
        layer_root: NodeId,
        position: Point,
        collect_stats: bool,
    ) -> (HitTestBoundsTreeQuery, HitTestBoundsTreeQueryStats) {
        let layer = self
            .view_boundaries
            .get(layer_root)
            .map(|boundary| &boundary.frame_products.hit_test_bounds);
        let hit_test_bounds_trees = &mut self.hit_test_bounds_trees;
        hit_test_bounds_trees.query(layer, position, collect_stats)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HitTestBoundsTreeQuery {
    Disabled,
    Miss,
    Hit(NodeId),
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct HitTestBoundsTreeQueryStats {
    pub(super) nodes_visited: u32,
    pub(super) nodes_pushed: u32,
}

#[derive(Debug)]
pub(in crate::tree) struct BoundaryHitTestBoundsState {
    frame_id: Option<FrameId>,
    product_initialized: bool,
    enabled: bool,
    tree: BoundsTree,
}

impl Default for BoundaryHitTestBoundsState {
    fn default() -> Self {
        Self {
            frame_id: None,
            product_initialized: false,
            enabled: false,
            tree: BoundsTree::default(),
        }
    }
}

impl BoundaryHitTestBoundsState {
    pub(in crate::tree) fn clear(&mut self) {
        self.frame_id = None;
        self.product_initialized = false;
        self.enabled = false;
        self.tree.clear_keep_alloc();
    }

    pub(in crate::tree) fn mark_unused(&mut self) {
        self.frame_id = None;
    }

    pub(in crate::tree) fn has_frame_product(&self) -> bool {
        self.frame_id.is_some()
    }

    fn can_reuse_for_stable_frame(&self) -> bool {
        self.product_initialized
    }

    fn enabled_for_frame(&self, frame_id: FrameId) -> bool {
        self.frame_id == Some(frame_id) && self.enabled
    }

    fn disable_for_frame(&mut self, frame_id: FrameId) {
        self.product_initialized = true;
        self.frame_id = Some(frame_id);
        self.enabled = false;
        self.tree.clear_keep_alloc();
    }

    fn take_tree_for_rebuild(&mut self, frame_id: FrameId) -> BoundsTree {
        self.product_initialized = true;
        self.frame_id = Some(frame_id);
        std::mem::take(&mut self.tree)
    }

    fn finish_rebuild(&mut self, frame_id: FrameId, enabled: bool, tree: BoundsTree) {
        self.product_initialized = true;
        self.frame_id = Some(frame_id);
        self.enabled = enabled;
        self.tree = tree;
    }

    pub(in crate::tree) fn reuse_for_frame(&mut self, frame_id: FrameId) {
        self.frame_id = Some(frame_id);
    }
}

#[derive(Debug, Default)]
struct BoundsTree {
    nodes: Vec<TreeNode>,
    root: Option<usize>,
    max_leaf: Option<usize>,
}

impl BoundsTree {
    fn clear_keep_alloc(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.max_leaf = None;
    }

    fn rebuild_from_leaves(
        &mut self,
        leaves: &mut [Leaf],
        level: &mut Vec<usize>,
        next_level: &mut Vec<usize>,
    ) {
        self.clear_keep_alloc();
        if leaves.is_empty() {
            return;
        }

        leaves.sort_by(|a, b| match a.center_x.total_cmp(&b.center_x) {
            Ordering::Equal => a.center_y.total_cmp(&b.center_y),
            ord => ord,
        });

        let mut max_leaf: Option<(u32, usize)> = None;
        for leaf in leaves.iter() {
            let idx = self.nodes.len();
            self.nodes.push(TreeNode {
                bounds: leaf.bounds,
                max_order: leaf.order,
                kind: TreeNodeKind::Leaf {
                    order: leaf.order,
                    node: leaf.node,
                },
            });
            match max_leaf {
                None => max_leaf = Some((leaf.order, idx)),
                Some((max, _)) if leaf.order > max => max_leaf = Some((leaf.order, idx)),
                _ => {}
            }
        }
        self.max_leaf = max_leaf.map(|(_, idx)| idx);

        level.clear();
        level.extend(0..self.nodes.len());

        while level.len() > 1 {
            next_level.clear();
            for chunk in level.chunks(MAX_CHILDREN) {
                let mut bounds = self.nodes[chunk[0]].bounds;
                let mut max_order = self.nodes[chunk[0]].max_order;
                for &child in &chunk[1..] {
                    bounds = rect_union(bounds, self.nodes[child].bounds);
                    max_order = max_order.max(self.nodes[child].max_order);
                }

                let mut children = NodeChildren::new();
                for &child in chunk {
                    children.push(child);
                }
                children.sort_by_max_order(&self.nodes);

                let idx = self.nodes.len();
                self.nodes.push(TreeNode {
                    bounds,
                    max_order,
                    kind: TreeNodeKind::Internal { children },
                });
                next_level.push(idx);
            }

            std::mem::swap(level, next_level);
        }

        self.root = level.first().copied();
    }

    fn find_max_containing_point(
        &self,
        point: Point,
        collect_stats: bool,
        search_stack: &mut Vec<usize>,
    ) -> (Option<NodeId>, HitTestBoundsTreeQueryStats) {
        let Some(root) = self.root else {
            return (None, HitTestBoundsTreeQueryStats::default());
        };

        let mut stats = HitTestBoundsTreeQueryStats::default();

        if let Some(max_idx) = self.max_leaf
            && rect_contains_point(self.nodes[max_idx].bounds, point)
        {
            if collect_stats {
                stats.nodes_visited = 1;
            }
            if let TreeNodeKind::Leaf { node, .. } = &self.nodes[max_idx].kind {
                return (Some(*node), stats);
            }
        }

        search_stack.clear();
        search_stack.push(root);
        if collect_stats {
            stats.nodes_pushed = stats.nodes_pushed.saturating_add(1);
        }

        let mut best_order: u32 = 0;
        let mut best_node: Option<NodeId> = None;

        while let Some(idx) = search_stack.pop() {
            if collect_stats {
                stats.nodes_visited = stats.nodes_visited.saturating_add(1);
            }
            let node = &self.nodes[idx];
            if node.max_order <= best_order {
                continue;
            }
            if !rect_contains_point(node.bounds, point) {
                continue;
            }

            match &node.kind {
                TreeNodeKind::Leaf { order, node } => {
                    if *order > best_order {
                        best_order = *order;
                        best_node = Some(*node);
                    }
                }
                TreeNodeKind::Internal { children } => {
                    // Children are sorted ascending by max_order. Push in-order so the highest
                    // max_order child is popped first.
                    for &child in children.as_slice() {
                        if self.nodes[child].max_order > best_order
                            && rect_contains_point(self.nodes[child].bounds, point)
                        {
                            search_stack.push(child);
                            if collect_stats {
                                stats.nodes_pushed = stats.nodes_pushed.saturating_add(1);
                            }
                        }
                    }
                }
            }
        }

        (best_node, stats)
    }
}

#[derive(Debug, Clone, Copy)]
struct Leaf {
    bounds: Rect,
    center_x: f32,
    center_y: f32,
    order: u32,
    node: NodeId,
}

#[derive(Debug, Clone)]
struct TreeNode {
    bounds: Rect,
    max_order: u32,
    kind: TreeNodeKind,
}

#[derive(Debug, Clone)]
enum TreeNodeKind {
    Leaf { order: u32, node: NodeId },
    Internal { children: NodeChildren },
}

#[derive(Debug, Clone)]
struct NodeChildren {
    indices: [usize; MAX_CHILDREN],
    len: u8,
}

impl NodeChildren {
    fn new() -> Self {
        Self {
            indices: [0; MAX_CHILDREN],
            len: 0,
        }
    }

    fn push(&mut self, index: usize) {
        debug_assert!((self.len as usize) < MAX_CHILDREN);
        self.indices[self.len as usize] = index;
        self.len += 1;
    }

    fn as_slice(&self) -> &[usize] {
        &self.indices[..self.len as usize]
    }

    fn as_mut_slice(&mut self) -> &mut [usize] {
        &mut self.indices[..self.len as usize]
    }

    fn sort_by_max_order(&mut self, nodes: &[TreeNode]) {
        self.as_mut_slice().sort_by_key(|idx| nodes[*idx].max_order);
    }
}

fn rect_is_empty(rect: Rect) -> bool {
    rect.size.width.0 <= 0.0 || rect.size.height.0 <= 0.0
}

fn rect_contains_point(rect: Rect, point: Point) -> bool {
    rect.contains(point)
}

fn rect_union(a: Rect, b: Rect) -> Rect {
    let (ax0, ay0, ax1, ay1) = rect_extents(a);
    let (bx0, by0, bx1, by1) = rect_extents(b);

    let x0 = ax0.min(bx0);
    let y0 = ay0.min(by0);
    let x1 = ax1.max(bx1);
    let y1 = ay1.max(by1);

    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px(x1 - x0), Px(y1 - y0)),
    )
}

fn rect_intersection(a: Rect, b: Rect) -> Rect {
    let (ax0, ay0, ax1, ay1) = rect_extents(a);
    let (bx0, by0, bx1, by1) = rect_extents(b);

    let x0 = ax0.max(bx0);
    let y0 = ay0.max(by0);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    if x1 <= x0 || y1 <= y0 {
        return Rect::new(Point::new(Px(x0), Px(y0)), Size::new(Px(0.0), Px(0.0)));
    }

    Rect::new(
        Point::new(Px(x0), Px(y0)),
        Size::new(Px(x1 - x0), Px(y1 - y0)),
    )
}

fn rect_extents(rect: Rect) -> (f32, f32, f32, f32) {
    let x0 = rect.origin.x.0;
    let y0 = rect.origin.y.0;
    let x1 = x0 + rect.size.width.0;
    let y1 = y0 + rect.size.height.0;
    (x0, y0, x1, y1)
}

fn transform_is_axis_aligned(t: Transform2D) -> bool {
    t.a.is_finite()
        && t.b.is_finite()
        && t.c.is_finite()
        && t.d.is_finite()
        && t.tx.is_finite()
        && t.ty.is_finite()
        && t.b == 0.0
        && t.c == 0.0
}

fn rect_transform_aabb(rect: Rect, t: Transform2D) -> Rect {
    if rect_is_empty(rect) {
        return rect;
    }

    let (x0, y0, x1, y1) = rect_extents(rect);
    let p00 = t.apply_point(Point::new(Px(x0), Px(y0)));
    let p10 = t.apply_point(Point::new(Px(x1), Px(y0)));
    let p01 = t.apply_point(Point::new(Px(x0), Px(y1)));
    let p11 = t.apply_point(Point::new(Px(x1), Px(y1)));

    let min_x = p00.x.0.min(p10.x.0).min(p01.x.0).min(p11.x.0);
    let max_x = p00.x.0.max(p10.x.0).max(p01.x.0).max(p11.x.0);
    let min_y = p00.y.0.min(p10.y.0).min(p01.y.0).min(p11.y.0);
    let max_y = p00.y.0.max(p10.y.0).max(p01.y.0).max(p11.y.0);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

fn rect_center_x(rect: Rect) -> f32 {
    let (x0, _, x1, _) = rect_extents(rect);
    (x0 + x1) * 0.5
}

fn rect_center_y(rect: Rect) -> f32 {
    let (_, y0, _, y1) = rect_extents(rect);
    (y0 + y1) * 0.5
}
