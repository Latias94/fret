use std::collections::HashMap;

use fret_core::{NodeId, Rect, Size};
use taffy::{TaffyTree, prelude::NodeId as TaffyNodeId};

use crate::layout_constraints::AvailableSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayoutId(TaffyNodeId);

#[derive(Default)]
pub struct TaffyLayoutEngine {
    tree: TaffyTree<()>,
    node_to_layout: HashMap<NodeId, LayoutId>,
    layout_to_node: HashMap<LayoutId, NodeId>,
}

impl TaffyLayoutEngine {
    pub fn begin_frame(&mut self) {}

    pub fn end_frame(&mut self) {}

    pub fn layout_id_for_node(&self, node: NodeId) -> Option<LayoutId> {
        self.node_to_layout.get(&node).copied()
    }

    pub fn node_for_layout_id(&self, id: LayoutId) -> Option<NodeId> {
        self.layout_to_node.get(&id).copied()
    }

    pub fn request_layout_node(&mut self, node: NodeId) -> LayoutId {
        if let Some(id) = self.node_to_layout.get(&node).copied() {
            return id;
        }
        let taffy_id = self.tree.new_leaf(Default::default()).expect("taffy new_leaf");
        let id = LayoutId(taffy_id);
        self.node_to_layout.insert(node, id);
        self.layout_to_node.insert(id, node);
        id
    }

    pub fn compute_root(
        &mut self,
        _root: LayoutId,
        _available: crate::layout_constraints::LayoutSize<AvailableSpace>,
    ) {
        // P1 skeleton: wiring lands in P2 (build/compute/apply).
    }

    pub fn layout_rect(&self, _id: LayoutId) -> Rect {
        Rect::new(fret_core::Point::new(fret_core::Px(0.0), fret_core::Px(0.0)), Size::default())
    }
}
