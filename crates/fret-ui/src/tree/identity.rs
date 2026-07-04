use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct StableNodeHandle {
    node: NodeId,
    binding_generation: u64,
}

impl StableNodeHandle {
    pub(crate) fn new(node: NodeId, binding_generation: u64) -> Self {
        Self {
            node,
            binding_generation,
        }
    }

    pub(crate) fn node(self) -> NodeId {
        self.node
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElementNodeIndexLiveLookup {
    Hit(NodeId),
    Missing,
    Stale,
    DuplicateLive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LiveTopologyEpoch(u64);

impl LiveTopologyEpoch {
    fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub(crate) fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Default)]
pub(crate) struct LiveTopologyIndex {
    live_layer_nodes: HashSet<NodeId>,
    child_parent: HashMap<NodeId, NodeId>,
    epoch: LiveTopologyEpoch,
}

impl LiveTopologyIndex {
    pub(crate) fn epoch(&self) -> LiveTopologyEpoch {
        self.epoch
    }

    fn advance_epoch(&mut self) {
        self.epoch = self.epoch.next();
    }

    fn clear(&mut self) {
        if self.live_layer_nodes.is_empty() && self.child_parent.is_empty() {
            return;
        }
        self.live_layer_nodes.clear();
        self.child_parent.clear();
        self.advance_epoch();
    }

    fn insert_live_node(&mut self, node: NodeId) {
        if self.live_layer_nodes.insert(node) {
            self.advance_epoch();
        }
    }

    fn remove_live_node(&mut self, node: NodeId) {
        if self.live_layer_nodes.remove(&node) {
            self.advance_epoch();
        }
    }

    fn contains_live_node(&self, node: NodeId) -> bool {
        self.live_layer_nodes.contains(&node)
    }

    fn child_parent(&self, child: NodeId) -> Option<NodeId> {
        self.child_parent.get(&child).copied()
    }

    fn set_child_parent_edge(&mut self, child: NodeId, parent: NodeId) {
        if self.child_parent.insert(child, parent) != Some(parent) {
            self.advance_epoch();
        }
    }

    fn remove_child_parent_edge_if_parent(&mut self, child: NodeId, parent: NodeId) {
        if self.child_parent.get(&child).copied() == Some(parent) {
            self.child_parent.remove(&child);
            self.advance_epoch();
        }
    }

    fn replace_child_parent_edges<H: UiHost>(
        &mut self,
        parent: NodeId,
        old_children: &[NodeId],
        new_children: &[NodeId],
        nodes: &SlotMap<NodeId, Node<H>>,
    ) {
        if old_children == new_children {
            for &child in new_children {
                if nodes.contains_key(child) {
                    self.set_child_parent_edge(child, parent);
                }
            }
            return;
        }

        for &child in old_children {
            self.remove_child_parent_edge_if_parent(child, parent);
        }
        for &child in new_children {
            if nodes.contains_key(child) {
                self.set_child_parent_edge(child, parent);
            }
        }
    }

    pub(in crate::tree) fn remove_node_and_direct_child_edges(
        &mut self,
        node: NodeId,
        direct_children: &[NodeId],
    ) {
        self.remove_live_node(node);
        if self.child_parent.remove(&node).is_some() {
            self.advance_epoch();
        }
        for &child in direct_children {
            self.remove_child_parent_edge_if_parent(child, node);
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ElementNodeIndex {
    by_element: HashMap<GlobalElementId, Vec<StableNodeHandle>>,
}

impl ElementNodeIndex {
    pub(crate) fn clear(&mut self) {
        self.by_element.clear();
    }

    pub(crate) fn bind(&mut self, element: GlobalElementId, handle: StableNodeHandle) {
        let handles = self.by_element.entry(element).or_default();
        handles.retain(|existing| existing.node != handle.node);
        handles.push(handle);
    }

    pub(crate) fn unbind_node(&mut self, element: GlobalElementId, node: NodeId) {
        let Some(handles) = self.by_element.get_mut(&element) else {
            return;
        };
        handles.retain(|handle| handle.node != node);
        if handles.is_empty() {
            self.by_element.remove(&element);
        }
    }

    pub(crate) fn handles_for(&self, element: GlobalElementId) -> Option<&[StableNodeHandle]> {
        self.by_element.get(&element).map(Vec::as_slice)
    }
}

impl<H: UiHost> UiTree<H> {
    pub(crate) fn stable_node_handle_for_node(&self, node: NodeId) -> Option<StableNodeHandle> {
        self.nodes
            .get(node)
            .map(|entry| StableNodeHandle::new(node, entry.element_binding_generation))
    }

    pub(in crate::tree) fn index_node_element_binding(
        &mut self,
        node: NodeId,
        element: GlobalElementId,
    ) {
        let Some(handle) = self.stable_node_handle_for_node(node) else {
            return;
        };
        self.element_node_index.bind(element, handle);
    }

    pub(in crate::tree) fn unindex_node_element_binding(
        &mut self,
        node: NodeId,
        element: GlobalElementId,
    ) {
        self.element_node_index.unbind_node(element, node);
    }

    pub(in crate::tree) fn index_live_subtree(&mut self, root: NodeId) {
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }
            let Some((element, children)) = self
                .nodes
                .get(node)
                .map(|entry| (entry.element, entry.children.clone()))
            else {
                continue;
            };
            self.live_topology.insert_live_node(node);
            if let Some(element) = element {
                self.index_node_element_binding(node, element);
            }
            for &child in &children {
                if self.nodes.contains_key(child) {
                    self.live_topology.set_child_parent_edge(child, node);
                }
            }
            stack.extend(children);
        }
    }

    pub(in crate::tree) fn unindex_detached_child_subtree(&mut self, root: NodeId) {
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack = vec![root];

        while let Some(node) = stack.pop() {
            if !visited.insert(node) {
                continue;
            }
            if self.root_to_layer.contains_key(&node) {
                continue;
            }
            let Some((element, children)) = self
                .nodes
                .get(node)
                .map(|entry| (entry.element, entry.children.clone()))
            else {
                self.live_topology
                    .remove_node_and_direct_child_edges(node, &[]);
                continue;
            };
            self.live_topology
                .remove_node_and_direct_child_edges(node, &children);
            if let Some(element) = element {
                self.unindex_node_element_binding(node, element);
            }
            stack.extend(children);
        }
    }

    pub(in crate::tree) fn rebuild_live_element_index(&mut self) {
        self.element_node_index.clear();
        self.live_topology.clear();
        for root in self.all_layer_roots() {
            self.index_live_subtree(root);
        }
    }

    pub(in crate::tree) fn replace_child_parent_index(
        &mut self,
        parent: NodeId,
        old_children: &[NodeId],
        new_children: &[NodeId],
    ) {
        self.live_topology.replace_child_parent_edges(
            parent,
            old_children,
            new_children,
            &self.nodes,
        );
    }

    pub(in crate::tree) fn node_is_reachable_from_layer_forest(&self, node: NodeId) -> bool {
        if !self.nodes.contains_key(node) {
            return false;
        }
        self.live_topology.contains_live_node(node)
    }

    pub(in crate::tree) fn parent_in_layer_forest_via_children(
        &self,
        node: NodeId,
    ) -> Option<NodeId> {
        if !self.nodes.contains_key(node) || self.root_to_layer.contains_key(&node) {
            return None;
        }

        if let Some(parent) = self.live_topology.child_parent(node)
            && self.live_topology.contains_live_node(parent)
            && self
                .nodes
                .get(parent)
                .is_some_and(|entry| entry.children.contains(&node))
        {
            return Some(parent);
        }

        let roots = self.all_layer_roots();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = roots;
        while let Some(parent) = stack.pop() {
            if !visited.insert(parent) {
                continue;
            }
            let Some(entry) = self.nodes.get(parent) else {
                continue;
            };
            for &child in &entry.children {
                if child == node {
                    return Some(parent);
                }
                stack.push(child);
            }
        }

        None
    }

    pub(in crate::tree) fn validated_child_edge_parent_for_reparent(
        &self,
        child: NodeId,
    ) -> Option<NodeId> {
        if let Some(parent) = self.live_topology.child_parent(child)
            && self
                .nodes
                .get(parent)
                .is_some_and(|entry| entry.children.contains(&child))
        {
            return Some(parent);
        }

        if self.live_topology.contains_live_node(child)
            && let Some(parent) = self.parent_in_layer_forest_via_children(child)
        {
            return Some(parent);
        }

        let retained_parent = self.nodes.get(child).and_then(|node| node.parent)?;
        self.nodes
            .get(retained_parent)
            .is_some_and(|entry| entry.children.contains(&child))
            .then_some(retained_parent)
    }

    pub fn node_parent_in_layer_tree(&self, node: NodeId) -> Option<NodeId> {
        self.parent_in_layer_forest_via_children(node)
    }

    pub(crate) fn live_topology_epoch(&self) -> LiveTopologyEpoch {
        self.live_topology.epoch()
    }

    fn stable_handle_matches_live_element(
        &self,
        element: GlobalElementId,
        handle: StableNodeHandle,
    ) -> Option<NodeId> {
        let node = handle.node();
        let entry = self.nodes.get(node)?;
        if entry.element != Some(element)
            || entry.element_binding_generation != handle.binding_generation
        {
            return None;
        }
        self.node_is_attached_to_layer_tree(node).then_some(node)
    }

    pub(crate) fn debug_record_index_duplicate_if_present(&mut self, element: GlobalElementId) {
        let live_count = self
            .element_node_index
            .handles_for(element)
            .map(|handles| {
                handles
                    .iter()
                    .filter(|&&handle| {
                        self.stable_handle_matches_live_element(element, handle)
                            .is_some()
                    })
                    .take(2)
                    .count()
            })
            .unwrap_or(0);
        if live_count > 1 {
            self.debug_record_identity_index_duplicate_live();
        }
    }

    pub(crate) fn resolve_indexed_live_attached_node_for_element(
        &mut self,
        element: GlobalElementId,
    ) -> ElementNodeIndexLiveLookup {
        let Some(handles) = self.element_node_index.handles_for(element) else {
            self.debug_record_identity_index_miss();
            return ElementNodeIndexLiveLookup::Missing;
        };

        let mut live: Option<NodeId> = None;
        let mut live_count: u32 = 0;
        for &handle in handles {
            if let Some(node) = self.stable_handle_matches_live_element(element, handle) {
                live_count = live_count.saturating_add(1);
                live.get_or_insert(node);
            }
        }

        match (live, live_count) {
            (Some(node), 1) => {
                self.debug_record_identity_index_hit();
                ElementNodeIndexLiveLookup::Hit(node)
            }
            (Some(_), _) => {
                self.debug_record_identity_index_duplicate_live();
                ElementNodeIndexLiveLookup::DuplicateLive
            }
            (None, _) => {
                self.debug_record_identity_index_stale();
                ElementNodeIndexLiveLookup::Stale
            }
        }
    }

    pub(in crate::tree) fn live_element_id_map(&self) -> HashMap<u64, NodeId> {
        let mut out = HashMap::with_capacity(self.element_node_index.by_element.len());
        for (&element, handles) in self.element_node_index.by_element.iter() {
            let mut live = None;
            let mut live_count: u32 = 0;
            for &handle in handles {
                if let Some(node) = self.stable_handle_matches_live_element(element, handle) {
                    live_count = live_count.saturating_add(1);
                    live.get_or_insert(node);
                    if live_count > 1 {
                        break;
                    }
                }
            }
            if live_count == 1
                && let Some(node) = live
            {
                out.insert(element.0, node);
            }
        }
        out
    }
}
