use super::*;
use std::collections::HashSet;

impl<H: UiHost> UiTree<H> {
    fn active_trapped_focus_scope_root(
        &self,
        app: &mut H,
        window: Option<AppWindowId>,
    ) -> Option<NodeId> {
        let window = window?;
        let mut node = self.focus?;
        loop {
            if let Some(record) = declarative::element_record_for_node(app, window, node)
                && matches!(
                    record.instance,
                    declarative::ElementInstance::FocusScope(p) if p.trap_focus
                )
            {
                return Some(node);
            }

            node = self.nodes.get(node).and_then(|n| n.parent)?;
        }
    }

    pub(in crate::tree) fn focus_request_is_allowed(
        &self,
        app: &mut H,
        window: Option<AppWindowId>,
        active_roots: &[NodeId],
        requested_focus: NodeId,
    ) -> bool {
        if self.focus == Some(requested_focus) {
            return false;
        }
        // Focus gating should be resilient to temporarily-broken parent pointers under retained /
        // view-cache-reused subtrees. Use reachability from active layer roots via child edges as
        // the authoritative layer membership check.
        if !self.is_reachable_from_any_root_via_children(requested_focus, active_roots) {
            return false;
        }

        let Some(trap_root) = self.active_trapped_focus_scope_root(app, window) else {
            return true;
        };
        self.is_descendant(trap_root, requested_focus)
    }

    fn is_reachable_from_any_root_via_children(&self, target: NodeId, roots: &[NodeId]) -> bool {
        if roots.is_empty() {
            return false;
        }
        if roots.contains(&target) {
            return true;
        }

        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = Vec::new();
        for &root in roots {
            if visited.insert(root) {
                stack.push(root);
            }
        }

        while let Some(node) = stack.pop() {
            let Some(entry) = self.nodes.get(node) else {
                continue;
            };
            for &child in &entry.children {
                if child == target {
                    return true;
                }
                if visited.insert(child) {
                    stack.push(child);
                }
            }
        }

        false
    }
}
