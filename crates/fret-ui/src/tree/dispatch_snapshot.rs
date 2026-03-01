use super::*;

/// A per-window, per-frame snapshot used to answer correctness-critical containment queries
/// (outside-press routing, focus containment, tab traversal) without depending on long-lived parent
/// pointers.
///
/// Phase C (workstream `ui-focus-overlay-fearless-refactor-v1`) builds on this to migrate routing
/// decisions to a coherent per-frame view.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct UiDispatchSnapshot {
    pub(crate) frame_id: FrameId,
    pub(crate) window: Option<AppWindowId>,
    pub(crate) active_layer_roots: Vec<NodeId>,
    pub(crate) barrier_root: Option<NodeId>,

    /// Nodes present in the snapshot forest in stable visit order (pre-order).
    pub(crate) nodes: Vec<NodeId>,

    /// Parent pointers for nodes that are present in this snapshot forest.
    pub(crate) parent: SecondaryMap<NodeId, Option<NodeId>>,

    /// DFS pre/post indices over the snapshot forest.
    ///
    /// These are only populated for nodes that are present in the snapshot forest.
    pub(crate) pre: SecondaryMap<NodeId, u32>,
    pub(crate) post: SecondaryMap<NodeId, u32>,
}

#[allow(dead_code)]
impl UiDispatchSnapshot {
    pub(crate) fn is_descendant(&self, root: NodeId, node: NodeId) -> bool {
        let (Some(&root_pre), Some(&root_post)) = (self.pre.get(root), self.post.get(root)) else {
            return false;
        };
        let (Some(&node_pre), Some(&node_post)) = (self.pre.get(node), self.post.get(node)) else {
            return false;
        };
        root_pre <= node_pre && node_post <= root_post
    }
}

impl<H: UiHost> UiTree<H> {
    /// Build a dispatch snapshot for the current window and `frame_id`.
    ///
    /// This is a mechanism-layer API. It does not change dispatch behavior by itself.
    #[allow(dead_code)]
    pub(in crate::tree) fn build_dispatch_snapshot(&self, frame_id: FrameId) -> UiDispatchSnapshot {
        let (active_layer_roots, barrier_root) = self.active_input_layers();

        let mut nodes: Vec<NodeId> = Vec::new();
        let mut parent: SecondaryMap<NodeId, Option<NodeId>> = SecondaryMap::new();
        let mut pre: SecondaryMap<NodeId, u32> = SecondaryMap::new();
        let mut post: SecondaryMap<NodeId, u32> = SecondaryMap::new();

        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut clock: u32 = 0;

        for &root in &active_layer_roots {
            if !self.nodes.contains_key(root) {
                continue;
            }
            if !visited.insert(root) {
                continue;
            }

            nodes.push(root);
            parent.insert(root, None);
            pre.insert(root, clock);
            clock = clock.saturating_add(1);

            let mut stack: Vec<(NodeId, usize)> = vec![(root, 0)];
            while let Some((node, next_child_index)) = stack.pop() {
                let Some(n) = self.nodes.get(node) else {
                    post.insert(node, clock);
                    clock = clock.saturating_add(1);
                    continue;
                };

                if next_child_index < n.children.len() {
                    // Resume this node after visiting the next child.
                    stack.push((node, next_child_index + 1));

                    let child = n.children[next_child_index];
                    if !self.nodes.contains_key(child) {
                        continue;
                    }
                    if !visited.insert(child) {
                        continue;
                    }

                    nodes.push(child);
                    parent.insert(child, Some(node));
                    pre.insert(child, clock);
                    clock = clock.saturating_add(1);
                    stack.push((child, 0));
                } else {
                    post.insert(node, clock);
                    clock = clock.saturating_add(1);
                }
            }
        }

        UiDispatchSnapshot {
            frame_id,
            window: self.window,
            active_layer_roots,
            barrier_root,
            nodes,
            parent,
            pre,
            post,
        }
    }
}
