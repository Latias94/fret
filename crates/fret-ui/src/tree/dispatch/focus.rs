use super::*;
use std::collections::HashSet;

impl<H: UiHost> UiTree<H> {
    fn active_trapped_focus_scope_root_in_snapshot(
        &self,
        app: &mut H,
        window: Option<AppWindowId>,
        snapshot: &UiDispatchSnapshot,
    ) -> Option<NodeId> {
        let window = window?;
        let mut node = self.focus?;
        if snapshot.pre.get(node).is_none() {
            return None;
        }

        loop {
            if let Some(record) = declarative::element_record_for_node(app, window, node)
                && matches!(
                    record.instance,
                    declarative::ElementInstance::FocusScope(p) if p.trap_focus
                )
            {
                return Some(node);
            }

            node = snapshot.parent.get(node).copied().flatten()?;
        }
    }

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
        snapshot: Option<&UiDispatchSnapshot>,
    ) -> bool {
        if self.focus == Some(requested_focus) {
            return false;
        }
        // Focus gating should be resilient to temporarily-broken parent pointers under retained /
        // view-cache-reused subtrees. Use reachability from active layer roots via child edges as
        // the authoritative layer membership check.
        let in_active_layers = if let Some(snapshot) = snapshot {
            snapshot.pre.get(requested_focus).is_some()
        } else {
            self.is_reachable_from_any_root_via_children(requested_focus, active_roots)
        };
        if !in_active_layers {
            return false;
        }

        let trap_root = if let Some(snapshot) = snapshot {
            // When a dispatch snapshot is available, avoid depending on retained parent pointers.
            self.active_trapped_focus_scope_root_in_snapshot(app, window, snapshot)
        } else {
            self.active_trapped_focus_scope_root(app, window)
        };

        let Some(trap_root) = trap_root else {
            return true;
        };

        if let Some(snapshot) = snapshot
            && snapshot.pre.get(trap_root).is_some()
            && snapshot.pre.get(requested_focus).is_some()
        {
            return snapshot.is_descendant(trap_root, requested_focus);
        }

        self.is_reachable_from_root_via_children(trap_root, requested_focus)
    }

    pub(in crate::tree) fn is_reachable_from_any_root_via_children(
        &self,
        target: NodeId,
        roots: &[NodeId],
    ) -> bool {
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

    pub(in crate::tree) fn is_reachable_from_root_via_children(
        &self,
        root: NodeId,
        target: NodeId,
    ) -> bool {
        if root == target {
            return true;
        }
        if !self.nodes.contains_key(root) || !self.nodes.contains_key(target) {
            return false;
        }

        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<NodeId> = Vec::new();
        visited.insert(root);
        stack.push(root);

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
