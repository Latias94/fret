use fret_ui::TreeNode;

#[derive(Debug, Clone)]
pub struct DemoHierarchy {
    pub roots: Vec<TreeNode>,
}

impl Default for DemoHierarchy {
    fn default() -> Self {
        let mut next_id: u64 = 1;
        let mut roots: Vec<TreeNode> = Vec::new();
        for r in 0..200u64 {
            let root_id = next_id;
            next_id += 1;

            let mut children: Vec<TreeNode> = Vec::new();
            for c in 0..20u64 {
                let child_id = next_id;
                next_id += 1;

                let mut grandchildren: Vec<TreeNode> = Vec::new();
                if c < 3 {
                    for g in 0..5u64 {
                        let grand_id = next_id;
                        next_id += 1;
                        grandchildren.push(TreeNode::new(
                            grand_id,
                            format!("Grandchild {r:03}-{c:02}-{g:02}"),
                        ));
                    }
                }

                children.push(
                    TreeNode::new(child_id, format!("Child {r:03}-{c:02}"))
                        .with_children(grandchildren),
                );
            }
            roots.push(TreeNode::new(root_id, format!("Root {r:03}")).with_children(children));
        }

        Self { roots }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyDropKind {
    InsertAbove,
    InsertBelow,
    ReparentInto,
    AppendRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HierarchyDropTarget {
    pub kind: HierarchyDropKind,
    pub target_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HierarchyMoveOp {
    pub node: u64,
    pub new_parent: Option<u64>,
    pub new_index: usize,
}

impl DemoHierarchy {
    pub fn next_available_id(&self) -> u64 {
        fn visit(nodes: &[TreeNode], max_id: &mut u64) {
            for n in nodes {
                *max_id = (*max_id).max(n.id);
                if !n.children.is_empty() {
                    visit(&n.children, max_id);
                }
            }
        }

        let mut max_id = 0u64;
        visit(&self.roots, &mut max_id);
        max_id.saturating_add(1)
    }

    pub fn create_entity(&mut self, parent: Option<u64>, label: String) -> u64 {
        let id = self.next_available_id();
        let node = TreeNode::new(id, label);

        match parent {
            None => {
                self.roots.push(node);
            }
            Some(parent) => {
                if let Some(p) = self.find_node_mut(parent) {
                    p.children.push(node);
                } else {
                    self.roots.push(node);
                }
            }
        }

        id
    }

    pub fn child_count(&self, parent: Option<u64>) -> usize {
        match parent {
            None => self.roots.len(),
            Some(parent) => self
                .find_node(parent)
                .map(|n| n.children.len())
                .unwrap_or(0),
        }
    }

    pub fn locate(&self, id: u64) -> Option<(Option<u64>, usize)> {
        fn visit(nodes: &[TreeNode], parent: Option<u64>, id: u64) -> Option<(Option<u64>, usize)> {
            for (i, n) in nodes.iter().enumerate() {
                if n.id == id {
                    return Some((parent, i));
                }
                if let Some(found) = visit(&n.children, Some(n.id), id) {
                    return Some(found);
                }
            }
            None
        }

        visit(&self.roots, None, id)
    }

    pub fn is_descendant_of(&self, ancestor: u64, maybe_descendant: u64) -> bool {
        let Some(node) = self.find_node(ancestor) else {
            return false;
        };
        fn contains(node: &TreeNode, id: u64) -> bool {
            for c in &node.children {
                if c.id == id || contains(c, id) {
                    return true;
                }
            }
            false
        }
        contains(node, maybe_descendant)
    }

    pub fn move_op_for_drop(
        &self,
        dragged: u64,
        drop: HierarchyDropTarget,
    ) -> Option<HierarchyMoveOp> {
        if dragged == drop.target_id.unwrap_or(dragged) {
            return None;
        }
        if let Some(target_id) = drop.target_id {
            if self.is_descendant_of(dragged, target_id) {
                return None;
            }
        }

        match drop.kind {
            HierarchyDropKind::AppendRoot => Some(HierarchyMoveOp {
                node: dragged,
                new_parent: None,
                new_index: self.roots.len(),
            }),
            HierarchyDropKind::ReparentInto => {
                let target = drop.target_id?;
                Some(HierarchyMoveOp {
                    node: dragged,
                    new_parent: Some(target),
                    new_index: self.child_count(Some(target)),
                })
            }
            HierarchyDropKind::InsertAbove | HierarchyDropKind::InsertBelow => {
                let target = drop.target_id?;
                let (parent, target_index) = self.locate(target)?;
                let insert_index = if drop.kind == HierarchyDropKind::InsertAbove {
                    target_index
                } else {
                    target_index + 1
                };
                Some(HierarchyMoveOp {
                    node: dragged,
                    new_parent: parent,
                    new_index: insert_index,
                })
            }
        }
    }

    pub fn apply_move(&mut self, op: HierarchyMoveOp) -> bool {
        let Some((old_parent, old_index)) = self.locate(op.node) else {
            return false;
        };

        if op.new_parent == Some(op.node) {
            return false;
        }
        if let Some(new_parent) = op.new_parent {
            if self.is_descendant_of(op.node, new_parent) {
                return false;
            }
            if self.find_node(new_parent).is_none() {
                return false;
            }
        }

        let node = match self.remove_node(op.node) {
            Some(n) => n,
            None => return false,
        };

        let mut index = op.new_index;
        if old_parent == op.new_parent && old_index < index {
            index = index.saturating_sub(1);
        }

        match op.new_parent {
            None => {
                let i = index.min(self.roots.len());
                self.roots.insert(i, node);
            }
            Some(parent) => {
                let p = self
                    .find_node_mut(parent)
                    .expect("parent existence checked before removal");
                let i = index.min(p.children.len());
                p.children.insert(i, node);
            }
        }

        true
    }

    fn find_node(&self, id: u64) -> Option<&TreeNode> {
        fn visit<'a>(nodes: &'a [TreeNode], id: u64) -> Option<&'a TreeNode> {
            for n in nodes {
                if n.id == id {
                    return Some(n);
                }
                if let Some(found) = visit(&n.children, id) {
                    return Some(found);
                }
            }
            None
        }

        visit(&self.roots, id)
    }

    fn find_node_mut(&mut self, id: u64) -> Option<&mut TreeNode> {
        fn visit<'a>(nodes: &'a mut [TreeNode], id: u64) -> Option<&'a mut TreeNode> {
            for n in nodes {
                if n.id == id {
                    return Some(n);
                }
                if let Some(found) = visit(&mut n.children, id) {
                    return Some(found);
                }
            }
            None
        }

        visit(&mut self.roots, id)
    }

    fn remove_node(&mut self, id: u64) -> Option<TreeNode> {
        fn remove_from(nodes: &mut Vec<TreeNode>, id: u64) -> Option<TreeNode> {
            if let Some(i) = nodes.iter().position(|n| n.id == id) {
                return Some(nodes.remove(i));
            }
            for n in nodes.iter_mut() {
                if let Some(found) = remove_from(&mut n.children, id) {
                    return Some(found);
                }
            }
            None
        }

        remove_from(&mut self.roots, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_entity_allocates_unique_id() {
        let mut h = DemoHierarchy::default();
        let id1 = h.create_entity(None, "A".to_string());
        let id2 = h.create_entity(None, "B".to_string());
        assert_ne!(id1, id2);
        assert!(id2 > id1);
    }

    #[test]
    fn create_entity_inserts_under_parent() {
        let mut h = DemoHierarchy::default();
        let parent = h.roots[0].id;
        let child = h.create_entity(Some(parent), "Child".to_string());
        let p = h.find_node(parent).expect("parent exists");
        assert!(p.children.iter().any(|c| c.id == child));
    }
}
