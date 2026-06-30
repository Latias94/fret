use super::*;
use slotmap::SecondaryMap;
use std::collections::HashSet;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn clear_all_semantics_dirty_tracking(&mut self) {
        for (_id, node) in self.nodes.iter_mut() {
            node.semantics_dirty = false;
            node.subtree_semantics_dirty_count = 0;
        }
    }

    pub(in crate::tree) fn clear_semantics_dirty_nodes(&mut self, nodes: Vec<NodeId>) {
        let mut seen: HashSet<NodeId> = HashSet::with_capacity(nodes.len());
        for node in nodes {
            if !seen.insert(node) {
                continue;
            }
            let Some(entry) = self.nodes.get_mut(node) else {
                continue;
            };
            if !entry.semantics_dirty {
                continue;
            }
            entry.semantics_dirty = false;
            self.apply_subtree_semantics_dirty_delta_to_node_and_ancestors(node, -1);
        }
    }

    pub(in crate::tree) fn apply_subtree_semantics_dirty_delta_to_node_and_ancestors(
        &mut self,
        mut current: NodeId,
        delta: i32,
    ) {
        let mut remaining = self.nodes.len().saturating_add(1);
        loop {
            if remaining == 0 {
                tracing::error!(
                    node = ?current,
                    "semantics dirty count propagation aborted (cycle or corrupt parent pointers?)"
                );
                break;
            }
            remaining = remaining.saturating_sub(1);

            let (parent, underflow) = {
                let Some(entry) = self.nodes.get_mut(current) else {
                    break;
                };
                let underflow =
                    apply_i32_delta_to_u32(&mut entry.subtree_semantics_dirty_count, delta);
                (entry.parent, underflow)
            };

            if underflow {
                tracing::error!(
                    node = ?current,
                    delta,
                    "subtree semantics dirty count underflow"
                );
                self.rebuild_subtree_semantics_dirty_counts_from(current);
                break;
            }

            let Some(parent) = parent else {
                break;
            };
            current = parent;
        }
    }

    fn rebuild_subtree_semantics_dirty_counts_from(&mut self, root: NodeId) {
        let root_parent = self.nodes.get(root).and_then(|n| n.parent);
        let old_root_count = self
            .nodes
            .get(root)
            .map(|n| n.subtree_semantics_dirty_count)
            .unwrap_or(0);

        let mut stack: Vec<(NodeId, bool)> = vec![(root, false)];
        while let Some((id, children_pushed)) = stack.pop() {
            let Some(entry) = self.nodes.get(id) else {
                continue;
            };
            if !children_pushed {
                stack.push((id, true));
                for &child in &entry.children {
                    stack.push((child, false));
                }
                continue;
            }

            let mut sum: u32 = if entry.semantics_dirty { 1 } else { 0 };
            for &child in &entry.children {
                sum = sum.saturating_add(
                    self.nodes
                        .get(child)
                        .map(|child| child.subtree_semantics_dirty_count)
                        .unwrap_or(0),
                );
            }
            if let Some(entry) = self.nodes.get_mut(id) {
                entry.subtree_semantics_dirty_count = sum;
            }
        }

        let new_root_count = self
            .nodes
            .get(root)
            .map(|n| n.subtree_semantics_dirty_count)
            .unwrap_or(0);
        let delta_i64 = new_root_count as i64 - old_root_count as i64;
        let delta = delta_i64.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
        self.apply_subtree_semantics_dirty_delta_to_ancestors(root_parent, delta);
    }

    fn apply_subtree_semantics_dirty_delta_to_ancestors(
        &mut self,
        mut current: Option<NodeId>,
        delta: i32,
    ) {
        if delta == 0 {
            return;
        }

        let mut remaining = self.nodes.len().saturating_add(1);
        while let Some(id) = current {
            if remaining == 0 {
                tracing::error!(
                    node = ?id,
                    "semantics dirty ancestor propagation aborted (cycle or corrupt parent pointers?)"
                );
                break;
            }
            remaining = remaining.saturating_sub(1);

            let (parent, underflow) = {
                let Some(entry) = self.nodes.get_mut(id) else {
                    break;
                };
                let underflow =
                    apply_i32_delta_to_u32(&mut entry.subtree_semantics_dirty_count, delta);
                (entry.parent, underflow)
            };
            if underflow {
                tracing::error!(node = ?id, delta, "subtree semantics dirty count underflow");
                self.rebuild_subtree_semantics_dirty_counts_from(id);
                break;
            }
            current = parent;
        }
    }

    pub(in crate::tree) fn set_layout_dirty_children_suppressed(
        &mut self,
        node: NodeId,
        suppressed: bool,
    ) {
        let changed = {
            let Some(entry) = self.nodes.get_mut(node) else {
                return;
            };
            if entry.layout_dirty_children_suppressed == suppressed {
                false
            } else {
                entry.layout_dirty_children_suppressed = suppressed;
                true
            }
        };

        if changed {
            self.recompute_node_subtree_layout_dirty_count_and_propagate(node);
        }
    }

    pub(crate) fn node_subtree_layout_dirty(&self, node: NodeId) -> bool {
        self.nodes
            .get(node)
            .is_some_and(|n| n.subtree_layout_dirty_count > 0)
    }

    #[allow(dead_code)]
    pub(crate) fn node_subtree_layout_dirty_count(&self, node: NodeId) -> u32 {
        self.nodes
            .get(node)
            .map(|n| n.subtree_layout_dirty_count)
            .unwrap_or(0)
    }

    pub(in crate::tree) fn node_layout_dirty_suppressed_by_ancestor(&self, node: NodeId) -> bool {
        let mut current = self.nodes.get(node).and_then(|n| n.parent);
        let mut remaining = self.nodes.len().saturating_add(1);
        while let Some(id) = current {
            if remaining == 0 {
                return false;
            }
            remaining = remaining.saturating_sub(1);
            let Some(entry) = self.nodes.get(id) else {
                return false;
            };
            if entry.layout_dirty_children_suppressed {
                return true;
            }
            current = entry.parent;
        }
        false
    }

    pub(crate) fn node_subtree_layout_dirty_covered_by_contained_view_cache_roots(
        &self,
        node: NodeId,
    ) -> bool {
        if !self.view_cache_active() || self.node_layout_invalidated(node) {
            return false;
        }

        let total = self.node_subtree_layout_dirty_count(node);
        if total == 0 || self.dirty_boundaries.is_empty() {
            return false;
        }

        let mut contained_roots: Vec<NodeId> = Vec::new();
        for &root in &self.dirty_boundaries {
            let Some(entry) = self.nodes.get(root) else {
                continue;
            };
            if !entry.view_cache.enabled
                || !entry.view_cache.layout_contained_when_bounds_known()
                || !entry.invalidation.layout
                || !self.node_is_descendant_or_self(root, node)
            {
                continue;
            }
            contained_roots.push(root);
        }

        if contained_roots.is_empty() {
            return false;
        }

        let mut covered = 0u32;
        for &root in &contained_roots {
            let nested_under_another_root = contained_roots
                .iter()
                .copied()
                .any(|other| other != root && self.node_is_descendant_or_self(root, other));
            if nested_under_another_root {
                continue;
            }
            covered = covered.saturating_add(self.node_subtree_layout_dirty_count(root));
        }

        covered == total
    }

    fn node_is_descendant_or_self(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = Some(node);
        let mut remaining = self.nodes.len().saturating_add(1);
        while let Some(id) = current {
            if remaining == 0 {
                return false;
            }
            remaining = remaining.saturating_sub(1);
            if id == ancestor {
                return true;
            }
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        false
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

        // Always keep boundary dirty roots discoverable, even if subtree aggregation is
        // disabled. Contained view-cache relayouts use `dirty_boundaries` as their entry set.
        if after
            && self.view_cache_active()
            && let Some(n) = self.nodes.get(node)
            && n.view_cache.enabled
            && n.view_cache.layout_contained_when_bounds_known()
        {
            self.mark_boundary_layout_dirty(
                node,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::SubtreeLayoutDirtyRepair,
            );
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
            && n.view_cache.layout_contained_when_bounds_known()
            && n.invalidation.layout
        {
            self.mark_boundary_layout_dirty(
                root,
                UiDebugInvalidationSource::Other,
                UiDebugInvalidationDetail::SubtreeLayoutDirtyRepair,
            );
        }

        let (parent, old_count, new_count) = {
            let Some(n) = self.nodes.get(node) else {
                return;
            };
            let mut sum: u32 = if n.invalidation.layout { 1 } else { 0 };
            if !n.layout_dirty_children_suppressed {
                for &child in &n.children {
                    sum = sum.saturating_add(
                        self.nodes
                            .get(child)
                            .map(|c| c.subtree_layout_dirty_count)
                            .unwrap_or(0),
                    );
                }
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
            if !n.layout_dirty_children_suppressed {
                for &child in &n.children {
                    sum = sum.saturating_add(
                        self.nodes
                            .get(child)
                            .map(|c| c.subtree_layout_dirty_count)
                            .unwrap_or(0),
                    );
                }
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
            if !n.layout_dirty_children_suppressed {
                for &child in &n.children {
                    sum = sum.saturating_add(
                        self.nodes
                            .get(child)
                            .map(|c| c.subtree_layout_dirty_count)
                            .unwrap_or(0),
                    );
                }
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
                if !n.layout_dirty_children_suppressed {
                    for &child in &n.children {
                        sum = sum.saturating_add(
                            self.nodes
                                .get(child)
                                .map(|c| c.subtree_layout_dirty_count)
                                .unwrap_or(0),
                        );
                    }
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

    #[track_caller]
    pub(in crate::tree) fn apply_subtree_layout_dirty_delta_to_node_and_ancestors(
        &mut self,
        node: NodeId,
        delta: i32,
    ) {
        self.apply_subtree_layout_dirty_delta_walk(Some(node), delta, true);
    }

    #[track_caller]
    pub(in crate::tree) fn apply_subtree_layout_dirty_child_delta_to_ancestors(
        &mut self,
        start: Option<NodeId>,
        delta: i32,
    ) {
        self.apply_subtree_layout_dirty_delta_walk(start, delta, false);
    }

    #[track_caller]
    fn apply_subtree_layout_dirty_delta_to_ancestors(&mut self, start: Option<NodeId>, delta: i32) {
        self.apply_subtree_layout_dirty_delta_walk(start, delta, false);
    }

    #[track_caller]
    fn apply_subtree_layout_dirty_delta_walk(
        &mut self,
        start: Option<NodeId>,
        delta: i32,
        include_start_self: bool,
    ) {
        if delta == 0 {
            return;
        }

        let mut walked_nodes: u32 = 0;
        let mut current = start;
        let mut first = true;
        while let Some(id) = current {
            let (parent, element, stored, underflow) = {
                let Some(n) = self.nodes.get_mut(id) else {
                    break;
                };

                if !(first && include_start_self) && n.layout_dirty_children_suppressed {
                    break;
                }

                let underflow = apply_i32_delta_to_u32(&mut n.subtree_layout_dirty_count, delta);
                (n.parent, n.element, n.subtree_layout_dirty_count, underflow)
            };
            if underflow {
                let caller = std::panic::Location::caller();
                tracing::error!(
                    node = ?id,
                    element = ?element,
                    stored,
                    delta,
                    caller = %caller,
                    "subtree layout dirty count underflow"
                );
                // Parent pointers participate in both delta propagation and repair walks. When an
                // underflow is observed, aggressively repair reachable parent pointers first so
                // the subsequent subtree-count rebuild can propagate along the most plausible
                // ancestry chain.
                let repaired_parents = self.repair_parent_pointers_from_layer_roots();
                self.debug_record_parent_pointer_repair(repaired_parents);
                if repaired_parents > 0 {
                    tracing::warn!(
                        node = ?id,
                        repaired_parents,
                        caller = %caller,
                        "repaired parent pointers after subtree layout dirty underflow"
                    );
                }
                self.repair_subtree_layout_dirty_counts_from(id);
                break;
            }
            walked_nodes = walked_nodes.saturating_add(1);
            current = parent;
            first = false;
        }

        if self.debug_enabled && walked_nodes > 0 {
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

    pub(in crate::tree) fn validate_subtree_layout_dirty_counts_if_enabled(&mut self) {
        let cfg = crate::runtime_config::ui_runtime_config();
        if !cfg.layout_subtree_dirty_aggregation_validate {
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
                if !n.layout_dirty_children_suppressed {
                    for &child in &n.children {
                        sum = sum.saturating_add(expected.get(child).copied().unwrap_or(0));
                    }
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
