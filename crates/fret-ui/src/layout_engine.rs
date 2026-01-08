use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use fret_core::{FrameId, NodeId, Point, Px, Rect, Size};
use taffy::{TaffyTree, prelude::NodeId as TaffyNodeId};

use crate::layout_constraints::{AvailableSpace, LayoutSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NodeContext {
    node: NodeId,
    measured: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutId(TaffyNodeId);

pub struct TaffyLayoutEngine {
    tree: TaffyTree<NodeContext>,
    node_to_layout: HashMap<NodeId, LayoutId>,
    layout_to_node: HashMap<LayoutId, NodeId>,
    children: HashMap<NodeId, Vec<NodeId>>,
    seen: HashSet<NodeId>,
    frame_id: Option<FrameId>,
    last_solve_time: Duration,
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        Self {
            tree: TaffyTree::new(),
            node_to_layout: HashMap::new(),
            layout_to_node: HashMap::new(),
            children: HashMap::new(),
            seen: HashSet::new(),
            frame_id: None,
            last_solve_time: Duration::default(),
        }
    }
}

impl TaffyLayoutEngine {
    pub fn begin_frame(&mut self, frame_id: FrameId) {
        if self.frame_id != Some(frame_id) {
            self.frame_id = Some(frame_id);
            self.seen.clear();
            self.last_solve_time = Duration::default();
        }
    }

    pub fn end_frame(&mut self) {
        let stale: Vec<NodeId> = self
            .node_to_layout
            .keys()
            .copied()
            .filter(|node| !self.seen.contains(node))
            .collect();

        for node in stale {
            let Some(layout_id) = self.node_to_layout.remove(&node) else {
                continue;
            };
            self.layout_to_node.remove(&layout_id);
            self.children.remove(&node);
            let _ = self.tree.remove(layout_id.0);
        }
        self.seen.clear();
    }

    pub fn layout_id_for_node(&self, node: NodeId) -> Option<LayoutId> {
        self.node_to_layout.get(&node).copied()
    }

    pub fn node_for_layout_id(&self, id: LayoutId) -> Option<NodeId> {
        self.layout_to_node.get(&id).copied()
    }

    pub fn request_layout_node(&mut self, node: NodeId) -> LayoutId {
        self.seen.insert(node);
        if let Some(id) = self.node_to_layout.get(&node).copied() {
            return id;
        }

        let taffy_id = self
            .tree
            .new_leaf_with_context(
                Default::default(),
                NodeContext {
                    node,
                    measured: false,
                },
            )
            .expect("taffy new_leaf");
        let id = LayoutId(taffy_id);
        self.node_to_layout.insert(node, id);
        self.layout_to_node.insert(id, node);
        id
    }

    pub fn set_children(&mut self, node: NodeId, children: &[NodeId]) {
        let parent = self.request_layout_node(node).0;

        let prev = self
            .children
            .get(&node)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if prev == children {
            return;
        }

        let mut child_nodes: Vec<TaffyNodeId> = Vec::with_capacity(children.len());
        for &child in children {
            let child_id = self.request_layout_node(child).0;
            child_nodes.push(child_id);
        }

        if self.tree.set_children(parent, &child_nodes).is_ok() {
            self.children.insert(node, children.to_vec());
            let _ = self.tree.mark_dirty(parent);
        }
    }

    pub fn compute_root(&mut self, root: LayoutId, available: LayoutSize<AvailableSpace>) {
        let started = Instant::now();

        let available = taffy::geometry::Size {
            width: match available.width {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
            height: match available.height {
                AvailableSpace::Definite(px) => taffy::style::AvailableSpace::Definite(px.0),
                AvailableSpace::MinContent => taffy::style::AvailableSpace::MinContent,
                AvailableSpace::MaxContent => taffy::style::AvailableSpace::MaxContent,
            },
        };

        self.tree
            .compute_layout_with_measure(root.0, available, |_known, _avail, _id, ctx, _style| {
                if ctx.is_some_and(|ctx| ctx.measured) {
                    // P2: measurement wiring lands after the two-phase protocol is in place.
                }
                taffy::geometry::Size::default()
            })
            .ok();

        self.last_solve_time += started.elapsed();
    }

    pub fn layout_rect(&self, id: LayoutId) -> Rect {
        let Ok(layout) = self.tree.layout(id.0) else {
            return Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default());
        };

        Rect::new(
            Point::new(Px(layout.location.x), Px(layout.location.y)),
            Size::new(Px(layout.size.width), Px(layout.size.height)),
        )
    }
}
