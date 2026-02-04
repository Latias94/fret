use super::*;

use std::cmp::Ordering;

const MAX_CHILDREN: usize = 12;
const DEFAULT_MIN_RECORDS: usize = 256;

fn hit_test_bounds_tree_disabled() -> bool {
    use std::sync::OnceLock;

    static DISABLED: OnceLock<bool> = OnceLock::new();
    *DISABLED.get_or_init(|| {
        std::env::var_os("FRET_UI_HIT_TEST_BOUNDS_TREE_DISABLE").is_some_and(|v| !v.is_empty())
    })
}

fn hit_test_bounds_tree_min_records() -> usize {
    use std::sync::OnceLock;

    static MIN_RECORDS: OnceLock<usize> = OnceLock::new();
    *MIN_RECORDS.get_or_init(|| {
        std::env::var("FRET_UI_HIT_TEST_BOUNDS_TREE_MIN_RECORDS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(DEFAULT_MIN_RECORDS)
            .max(1)
    })
}

#[derive(Debug, Default)]
pub(super) struct HitTestBoundsTrees {
    frame_id: Option<FrameId>,
    layers: Vec<LayerBoundsTree>,
    clip_stack: Vec<(NodeId, Option<Rect>, Transform2D)>,
    leaves: Vec<Leaf>,
}

impl HitTestBoundsTrees {
    pub(super) fn clear(&mut self) {
        self.frame_id = None;
        for layer in &mut self.layers {
            layer.used_this_frame = false;
            layer.enabled = false;
            layer.tree.clear_keep_alloc();
        }
    }

    pub(super) fn begin_frame(&mut self, frame_id: FrameId) {
        if hit_test_bounds_tree_disabled() {
            self.clear();
            return;
        }

        self.frame_id = Some(frame_id);
        for layer in &mut self.layers {
            layer.used_this_frame = false;
        }
    }

    pub(super) fn rebuild_for_layer_from_records<H: UiHost>(
        &mut self,
        layer_root: NodeId,
        records: &[prepaint::InteractionRecord],
        nodes: &SlotMap<NodeId, Node<H>>,
    ) {
        if hit_test_bounds_tree_disabled() {
            return;
        }
        if records.len() < hit_test_bounds_tree_min_records() {
            let layer = self.layer_mut(layer_root);
            layer.enabled = false;
            layer.tree.clear_keep_alloc();
            return;
        }
        if records.first().is_none_or(|r| r.node != layer_root) {
            let layer = self.layer_mut(layer_root);
            layer.enabled = false;
            layer.tree.clear_keep_alloc();
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

        let mut tree = {
            let layer = self.layer_mut(layer_root);
            std::mem::take(&mut layer.tree)
        };

        if disabled_or_empty {
            tree.clear_keep_alloc();
            let layer = self.layer_mut(layer_root);
            layer.enabled = false;
            layer.tree = tree;
            return;
        }

        tree.rebuild_from_leaves(&mut self.leaves);
        let layer = self.layer_mut(layer_root);
        layer.enabled = true;
        layer.tree = tree;
    }

    pub(super) fn query(
        &mut self,
        layer_root: NodeId,
        position: Point,
        collect_stats: bool,
    ) -> (HitTestBoundsTreeQuery, HitTestBoundsTreeQueryStats) {
        let Some(frame_id) = self.frame_id else {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        };
        let layer = self
            .layers
            .iter_mut()
            .find(|l| l.used_this_frame && l.frame_id == Some(frame_id) && l.root == layer_root);
        let Some(layer) = layer else {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        };
        if !layer.enabled {
            return (
                HitTestBoundsTreeQuery::Disabled,
                HitTestBoundsTreeQueryStats::default(),
            );
        }
        let (hit, stats) = layer
            .tree
            .find_max_containing_point(position, collect_stats);
        let query = match hit {
            Some(hit) => HitTestBoundsTreeQuery::Hit(hit),
            None => HitTestBoundsTreeQuery::Miss,
        };
        (query, stats)
    }

    pub(super) fn reuse_for_layer(&mut self, layer_root: NodeId) {
        if hit_test_bounds_tree_disabled() {
            return;
        }

        let Some(frame_id) = self.frame_id else {
            return;
        };

        if let Some(layer) = self.layers.iter_mut().find(|l| l.root == layer_root) {
            layer.used_this_frame = true;
            layer.frame_id = Some(frame_id);
            return;
        }

        let mut layer = LayerBoundsTree::new(layer_root);
        layer.used_this_frame = true;
        layer.frame_id = Some(frame_id);
        self.layers.push(layer);
    }

    fn layer_mut(&mut self, layer_root: NodeId) -> &mut LayerBoundsTree {
        let frame_id = self.frame_id;

        let idx = match self.layers.iter().position(|l| l.root == layer_root) {
            Some(idx) => idx,
            None => match self.layers.iter().position(|l| !l.used_this_frame) {
                Some(reuse) => {
                    self.layers[reuse].root = layer_root;
                    reuse
                }
                None => {
                    self.layers.push(LayerBoundsTree::new(layer_root));
                    self.layers.len() - 1
                }
            },
        };

        let layer = &mut self.layers[idx];
        layer.used_this_frame = true;
        layer.frame_id = frame_id;
        layer
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
struct LayerBoundsTree {
    root: NodeId,
    frame_id: Option<FrameId>,
    used_this_frame: bool,
    enabled: bool,
    tree: BoundsTree,
}

impl LayerBoundsTree {
    fn new(root: NodeId) -> Self {
        Self {
            root,
            frame_id: None,
            used_this_frame: false,
            enabled: false,
            tree: BoundsTree::default(),
        }
    }
}

#[derive(Debug, Default)]
struct BoundsTree {
    nodes: Vec<TreeNode>,
    root: Option<usize>,
    max_leaf: Option<usize>,
    search_stack: Vec<usize>,
    level: Vec<usize>,
    next_level: Vec<usize>,
}

impl BoundsTree {
    fn clear_keep_alloc(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.max_leaf = None;
        self.search_stack.clear();
        self.level.clear();
        self.next_level.clear();
    }

    fn rebuild_from_leaves(&mut self, leaves: &mut [Leaf]) {
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

        self.level.clear();
        self.level.extend(0..self.nodes.len());

        while self.level.len() > 1 {
            self.next_level.clear();
            for chunk in self.level.chunks(MAX_CHILDREN) {
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
                self.next_level.push(idx);
            }

            std::mem::swap(&mut self.level, &mut self.next_level);
        }

        self.root = self.level.first().copied();
    }

    fn find_max_containing_point(
        &mut self,
        point: Point,
        collect_stats: bool,
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

        self.search_stack.clear();
        self.search_stack.push(root);
        if collect_stats {
            stats.nodes_pushed = stats.nodes_pushed.saturating_add(1);
        }

        let mut best_order: u32 = 0;
        let mut best_node: Option<NodeId> = None;

        while let Some(idx) = self.search_stack.pop() {
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
                            self.search_stack.push(child);
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
