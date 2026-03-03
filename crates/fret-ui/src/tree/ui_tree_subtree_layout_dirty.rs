use super::*;
use slotmap::SecondaryMap;
use std::collections::HashSet;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn subtree_layout_dirty_aggregation_enabled(&self) -> bool {
        crate::runtime_config::ui_runtime_config().layout_subtree_dirty_aggregation
    }

    pub(crate) fn node_subtree_layout_dirty(&self, node: NodeId) -> bool {
        self.subtree_layout_dirty_aggregation_enabled()
            && self
                .nodes
                .get(node)
                .is_some_and(|n| n.subtree_layout_dirty_count > 0)
    }

    #[allow(dead_code)]
    pub(crate) fn node_subtree_layout_dirty_count(&self, node: NodeId) -> u32 {
        if !self.subtree_layout_dirty_aggregation_enabled() {
            return 0;
        }
        self.nodes
            .get(node)
            .map(|n| n.subtree_layout_dirty_count)
            .unwrap_or(0)
    }

    pub(in crate::tree) fn note_layout_invalidation_transition_for_subtree_aggregation(
        &mut self,
        node: NodeId,
        before: bool,
        after: bool,
    ) {
        if before == after {
            return;
        }

        // Always keep view-cache dirty roots discoverable, even if subtree aggregation is
        // disabled. Contained view-cache relayouts use `dirty_cache_roots` as their entry set.
        if after
            && self.view_cache_active()
            && let Some(n) = self.nodes.get(node)
            && n.view_cache.enabled
            && n.view_cache.contained_layout
        {
            self.mark_cache_root_dirty(
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::Unknown,
            );
        }

        if !self.subtree_layout_dirty_aggregation_enabled() {
            return;
        }

        let delta: i32 = if after { 1 } else { -1 };
        self.apply_subtree_layout_dirty_delta_to_node_and_ancestors(node, delta);
    }

    pub(in crate::tree) fn recompute_node_subtree_layout_dirty_count_and_propagate(
        &mut self,
        node: NodeId,
    ) {
        // Always keep view-cache dirty roots discoverable, even if subtree aggregation is
        // disabled. This mirrors the older retained-tree behavior and is relied on by mutation
        // paths that toggle invalidations without a full invalidation walk.
        if self.view_cache_active()
            && let Some(root) = self.nearest_view_cache_root(node)
            && let Some(n) = self.nodes.get(root)
            && n.view_cache.enabled
            && n.view_cache.contained_layout
            && n.invalidation.layout
        {
            self.mark_cache_root_dirty(
                root,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::Unknown,
            );
        }

        if !self.subtree_layout_dirty_aggregation_enabled() {
            return;
        }

        let (parent, old_count, new_count) = {
            let Some(n) = self.nodes.get(node) else {
                return;
            };
            let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
            for &child in &n.children {
                sum = sum.saturating_add(
                    self.nodes
                        .get(child)
                        .map(|c| c.subtree_layout_dirty_count)
                        .unwrap_or(0),
                );
            }
            (n.parent, n.subtree_layout_dirty_count, sum)
        };

        if old_count == new_count {
            return;
        }

        if let Some(n) = self.nodes.get_mut(node) {
            n.subtree_layout_dirty_count = new_count;
        }

        let delta_i64: i64 = new_count as i64 - old_count as i64;
        debug_assert!(delta_i64 >= i32::MIN as i64 && delta_i64 <= i32::MAX as i64);
        let delta: i32 = delta_i64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.apply_subtree_layout_dirty_delta_to_ancestors(parent, delta);
    }

    pub(in crate::tree) fn rebuild_subtree_layout_dirty_counts_and_propagate(
        &mut self,
        root: NodeId,
    ) {
        if !self.subtree_layout_dirty_aggregation_enabled() {
            return;
        }

        let root_parent = self.nodes.get(root).and_then(|n| n.parent);
        let old_root_count = self
            .nodes
            .get(root)
            .map(|n| n.subtree_layout_dirty_count)
            .unwrap_or(0);

        let mut stack: Vec<(NodeId, bool)> = Vec::new();
        stack.push((root, false));
        let mut rebuilt_nodes: u32 = 0;
        while let Some((id, children_pushed)) = stack.pop() {
            let Some(n) = self.nodes.get(id) else {
                continue;
            };
            if !children_pushed {
                stack.push((id, true));
                for &child in &n.children {
                    stack.push((child, false));
                }
                continue;
            }

            let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
            for &child in &n.children {
                sum = sum.saturating_add(
                    self.nodes
                        .get(child)
                        .map(|c| c.subtree_layout_dirty_count)
                        .unwrap_or(0),
                );
            }
            if let Some(n) = self.nodes.get_mut(id) {
                n.subtree_layout_dirty_count = sum;
            }
            rebuilt_nodes = rebuilt_nodes.saturating_add(1);
        }

        if self.debug_enabled {
            self.debug_stats.layout_subtree_dirty_agg_rebuild_nodes = self
                .debug_stats
                .layout_subtree_dirty_agg_rebuild_nodes
                .saturating_add(rebuilt_nodes);
        }

        let new_root_count = self
            .nodes
            .get(root)
            .map(|n| n.subtree_layout_dirty_count)
            .unwrap_or(0);
        let delta_i64: i64 = new_root_count as i64 - old_root_count as i64;
        debug_assert!(delta_i64 >= i32::MIN as i64 && delta_i64 <= i32::MAX as i64);
        let delta: i32 = delta_i64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.apply_subtree_layout_dirty_delta_to_ancestors(root_parent, delta);
    }

    pub(in crate::tree) fn repair_subtree_layout_dirty_counts_from(&mut self, root: NodeId) {
        if !self.subtree_layout_dirty_aggregation_enabled() {
            return;
        }

        // Step 1: rebuild the subtree rooted at `root` (post-order) so descendants become
        // internally consistent with their invalidation flags and child lists.
        let mut stack: Vec<(NodeId, bool)> = Vec::new();
        stack.push((root, false));
        let mut rebuilt_nodes: u32 = 0;
        while let Some((id, children_pushed)) = stack.pop() {
            let Some(n) = self.nodes.get(id) else {
                continue;
            };
            if !children_pushed {
                stack.push((id, true));
                for &child in &n.children {
                    stack.push((child, false));
                }
                continue;
            }

            let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
            for &child in &n.children {
                sum = sum.saturating_add(
                    self.nodes
                        .get(child)
                        .map(|c| c.subtree_layout_dirty_count)
                        .unwrap_or(0),
                );
            }
            if let Some(n) = self.nodes.get_mut(id) {
                n.subtree_layout_dirty_count = sum;
            }
            rebuilt_nodes = rebuilt_nodes.saturating_add(1);
        }

        // Step 2: recompute exact counts on ancestors so drift cannot linger above `root` even if
        // the previously stored `root` count (used by delta propagation) was already incorrect.
        let mut walked_nodes: u32 = 0;
        let mut current = self.nodes.get(root).and_then(|n| n.parent);
        while let Some(id) = current {
            let (next_parent, expected) = {
                let Some(n) = self.nodes.get(id) else {
                    break;
                };
                let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
                for &child in &n.children {
                    sum = sum.saturating_add(
                        self.nodes
                            .get(child)
                            .map(|c| c.subtree_layout_dirty_count)
                            .unwrap_or(0),
                    );
                }
                (n.parent, sum)
            };

            if let Some(n) = self.nodes.get_mut(id) {
                n.subtree_layout_dirty_count = expected;
            }

            walked_nodes = walked_nodes.saturating_add(1);
            if walked_nodes > 4096 {
                tracing::warn!(
                    node = ?id,
                    "repair_subtree_layout_dirty_counts_from: aborting ancestor walk (cycle or corrupt parent pointers?)"
                );
                break;
            }
            current = next_parent;
        }

        if self.debug_enabled {
            self.debug_stats.layout_subtree_dirty_agg_rebuild_nodes = self
                .debug_stats
                .layout_subtree_dirty_agg_rebuild_nodes
                .saturating_add(rebuilt_nodes);
        }
    }

    pub(in crate::tree) fn apply_subtree_layout_dirty_delta_to_node_and_ancestors(
        &mut self,
        node: NodeId,
        delta: i32,
    ) {
        if delta == 0 || !self.subtree_layout_dirty_aggregation_enabled() {
            return;
        }

        let mut walked_nodes: u32 = 0;
        let mut current = Some(node);
        while let Some(id) = current {
            let (parent, element, stored, underflow) = {
                let Some(n) = self.nodes.get_mut(id) else {
                    break;
                };
                let underflow = apply_i32_delta_to_u32(&mut n.subtree_layout_dirty_count, delta);
                (n.parent, n.element, n.subtree_layout_dirty_count, underflow)
            };
            if underflow {
                tracing::error!(
                    node = ?id,
                    element = ?element,
                    stored,
                    delta,
                    "subtree layout dirty count underflow"
                );
                self.repair_subtree_layout_dirty_counts_from(id);
                break;
            }
            walked_nodes = walked_nodes.saturating_add(1);
            current = parent;
        }

        if self.debug_enabled {
            self.debug_stats.layout_subtree_dirty_agg_updates = self
                .debug_stats
                .layout_subtree_dirty_agg_updates
                .saturating_add(1);
            self.debug_stats.layout_subtree_dirty_agg_nodes_touched = self
                .debug_stats
                .layout_subtree_dirty_agg_nodes_touched
                .saturating_add(walked_nodes);
            self.debug_stats.layout_subtree_dirty_agg_max_parent_walk = self
                .debug_stats
                .layout_subtree_dirty_agg_max_parent_walk
                .max(walked_nodes);
        }
    }

    fn apply_subtree_layout_dirty_delta_to_ancestors(&mut self, start: Option<NodeId>, delta: i32) {
        let Some(node) = start else {
            return;
        };
        self.apply_subtree_layout_dirty_delta_to_node_and_ancestors(node, delta);
    }

    pub(in crate::tree) fn validate_subtree_layout_dirty_counts_if_enabled(&mut self) {
        let cfg = crate::runtime_config::ui_runtime_config();
        if !cfg.layout_subtree_dirty_aggregation_validate {
            return;
        }
        if !cfg.layout_subtree_dirty_aggregation {
            return;
        }

        let mut expected: SecondaryMap<NodeId, u32> = SecondaryMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(NodeId, bool)> = Vec::new();

        for (root, _) in self.nodes.iter() {
            if !visited.insert(root) {
                continue;
            }
            stack.push((root, false));
            while let Some((id, children_pushed)) = stack.pop() {
                let Some(n) = self.nodes.get(id) else {
                    continue;
                };
                if !children_pushed {
                    stack.push((id, true));
                    for &child in &n.children {
                        if visited.insert(child) {
                            stack.push((child, false));
                        }
                    }
                    continue;
                }

                let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
                for &child in &n.children {
                    sum = sum.saturating_add(expected.get(child).copied().unwrap_or(0));
                }
                expected.insert(id, sum);
            }
        }

        let mut failures: u32 = 0;
        const MAX_REPORTS: usize = 16;
        for (id, n) in self.nodes.iter() {
            let exp = expected.get(id).copied().unwrap_or(0);
            if n.subtree_layout_dirty_count == exp {
                continue;
            }
            failures = failures.saturating_add(1);
            if (failures as usize) <= MAX_REPORTS {
                tracing::error!(
                    node = ?id,
                    element = ?n.element,
                    stored = n.subtree_layout_dirty_count,
                    expected = exp,
                    "subtree layout dirty count drift"
                );
            }
        }

        if failures == 0 {
            return;
        }

        if self.debug_enabled {
            self.debug_stats
                .layout_subtree_dirty_agg_validation_failures = self
                .debug_stats
                .layout_subtree_dirty_agg_validation_failures
                .saturating_add(failures);
        }

        if cfg.layout_subtree_dirty_aggregation_validate_panic {
            panic!("subtree layout dirty count drift: failures={failures}");
        }
    }
}

pub(in crate::tree) fn apply_i32_delta_to_u32(value: &mut u32, delta: i32) -> bool {
    if delta > 0 {
        *value = value.saturating_add(delta as u32);
        return false;
    }
    if delta < 0 {
        let dec = (-delta) as u32;
        if *value < dec {
            return true;
        }
        *value -= dec;
    }
    false
}
