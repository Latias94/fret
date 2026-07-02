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
            if let Some(element) = element {
                self.index_node_element_binding(node, element);
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
                continue;
            };
            if let Some(element) = element {
                self.unindex_node_element_binding(node, element);
            }
            stack.extend(children);
        }
    }

    pub(in crate::tree) fn rebuild_live_element_index(&mut self) {
        self.element_node_index.clear();
        for root in self.all_layer_roots() {
            self.index_live_subtree(root);
        }
    }

    pub(in crate::tree) fn node_is_reachable_from_layer_forest(&self, node: NodeId) -> bool {
        if !self.nodes.contains_key(node) {
            return false;
        }
        if self.root_to_layer.contains_key(&node) {
            return true;
        }
        let roots = self.all_layer_roots();
        self.is_reachable_from_any_root_via_children(node, roots.as_slice())
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
