use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn set_paint_cache_policy(&mut self, policy: PaintCachePolicy) {
        self.paint_cache_policy = policy;
    }

    pub fn paint_cache_policy(&self) -> PaintCachePolicy {
        self.paint_cache_policy
    }

    pub fn set_view_cache_enabled(&mut self, enabled: bool) {
        self.view_cache_enabled = enabled;
    }

    pub fn view_cache_enabled(&self) -> bool {
        self.view_cache_enabled
    }

    pub fn set_inspection_active(&mut self, active: bool) {
        self.inspection_active = active;
    }

    pub fn inspection_active(&self) -> bool {
        self.inspection_active
    }

    pub fn set_paint_cache_enabled(&mut self, enabled: bool) {
        self.set_paint_cache_policy(if enabled {
            PaintCachePolicy::Enabled
        } else {
            PaintCachePolicy::Disabled
        });
    }

    pub fn paint_cache_enabled(&self) -> bool {
        match self.paint_cache_policy {
            PaintCachePolicy::Auto => !self.inspection_active,
            PaintCachePolicy::Enabled => true,
            PaintCachePolicy::Disabled => false,
        }
    }

    /// Ingest the previous frame's recorded ops from `scene` for paint-cache replay.
    ///
    /// Call this **before** clearing `scene` for the next frame.
    ///
    /// Important:
    /// - This method is destructive: it swaps the scene op storage into the UI tree. Do not call
    ///   it more than once for the same `Scene` before `Scene::clear()`.
    /// - `scene` must contain the previous frame ops that were produced by **this** `UiTree`.
    /// - The paint cache records absolute op index ranges into the previous frame ops vector, so
    ///   sharing a single `Scene` across multiple `UiTree`s is not compatible with paint-cache
    ///   ingestion unless each tree records into an isolated scene.
    pub fn ingest_paint_cache_source(&mut self, scene: &mut Scene) {
        scene.swap_storage(
            &mut self.paint_cache.prev_ops,
            &mut self.paint_cache.prev_fingerprint,
        );
    }

    pub(in crate::tree) fn view_cache_active(&self) -> bool {
        self.view_cache_enabled && !self.inspection_active
    }

    pub(in crate::tree) fn nearest_view_cache_root(&self, node: NodeId) -> Option<NodeId> {
        let mut current = Some(node);
        while let Some(id) = current {
            let n = self.nodes.get(id)?;
            if n.view_cache.enabled {
                return Some(id);
            }
            current = n.parent;
        }
        None
    }

    pub(in crate::tree) fn mark_cache_root_dirty(
        &mut self,
        root: NodeId,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        self.dirty_cache_roots.insert(root);
        self.dirty_cache_root_reasons.insert(root, (source, detail));
    }

    pub(crate) fn should_reuse_view_cache_node(&self, node: NodeId) -> bool {
        if !self.view_cache_active() {
            return false;
        }
        let Some(n) = self.nodes.get(node) else {
            return false;
        };
        if !n.view_cache.enabled {
            return false;
        }
        if n.view_cache_needs_rerender {
            return false;
        }
        // View-cache reuse is an authoring-level "skip re-render" decision, not a "skip repaint"
        // decision: paint invalidations (e.g. hover/focus) should not force a child render pass.
        if !n.invalidation.layout {
            return true;
        }

        // Layout invalidations are only safe to ignore for cache roots that opt into contained
        // layout behavior with definite (non-auto) sizing and known bounds.
        //
        // This mirrors the same conditions used by invalidation propagation to truncate at cache
        // boundaries.
        n.view_cache.contained_layout
            && n.view_cache.layout_definite
            && n.bounds.size != Size::default()
    }

    pub(crate) fn view_cache_node_needs_rerender(&self, node: NodeId) -> bool {
        self.nodes
            .get(node)
            .is_some_and(|n| n.view_cache_needs_rerender)
    }

    /// Configure view-cache behavior for a specific node.
    ///
    /// This is an advanced/low-level knob. Most applications should prefer declarative
    /// view-cache boundaries, but retained widgets (and diagnostics harnesses) may need to enable
    /// view caching explicitly on a node.
    pub fn set_node_view_cache_flags(
        &mut self,
        node: NodeId,
        enabled: bool,
        contained_layout: bool,
        layout_definite: bool,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            let next = ViewCacheFlags {
                enabled,
                contained_layout,
                layout_definite,
            };
            if n.view_cache == next {
                return;
            }
            n.view_cache = next;
        }
    }

    pub(crate) fn set_node_view_cache_needs_rerender(&mut self, node: NodeId, needs: bool) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.view_cache_needs_rerender = needs;
        }
        if !needs {
            self.dirty_cache_roots.remove(&node);
            self.dirty_cache_root_reasons.remove(&node);
        }
    }

    /// Mark the nearest view-cache root as "needs rerender" without forcing a layout invalidation walk.
    ///
    /// This is intended for barrier-driven widgets (virtual lists, scroll content, etc.) that can
    /// detect a logical "window mismatch" during layout and need the *next frame* to rerun the
    /// declarative render closure to rebuild children, but do not benefit from triggering an
    /// additional contained relayout pass in the *current* frame.
    pub(crate) fn mark_nearest_view_cache_root_needs_rerender(
        &mut self,
        node: NodeId,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        if !self.view_cache_active() {
            return;
        }

        if !Self::invalidation_marks_view_dirty(source, Invalidation::HitTestOnly, detail) {
            return;
        }

        let Some(root) = self.nearest_view_cache_root(node) else {
            return;
        };

        let mut current: Option<NodeId> = Some(root);
        while let Some(id) = current {
            let next_parent = self.nodes.get(id).and_then(|n| n.parent);
            if let Some(n) = self.nodes.get_mut(id)
                && n.view_cache.enabled
            {
                n.view_cache_needs_rerender = true;
                self.mark_cache_root_dirty(id, source, detail);
            }
            current = next_parent;
        }
    }

    /// Repair invalidation propagation for newly mounted auto-sized cache roots.
    ///
    /// During declarative mounting we may discover `ViewCache` roots before their parent pointers
    /// are fully connected. When view caching is active, invalidation propagation can be
    /// truncated at cache roots, and a cache root that is only marked dirty on itself may never be
    /// laid out by its (still-clean) ancestors. This shows up as cache-root subtrees stuck at
    /// `Rect::default()` origins (e.g. scripted clicks using semantics bounds land in the wrong
    /// place).
    ///
    /// Call this after `repair_parent_pointers_from_layer_roots()` and before `layout_all` so the
    /// next layout pass walks far enough to place newly mounted cache-root subtrees.
    pub(crate) fn propagate_auto_sized_view_cache_root_invalidations(&mut self) {
        if !self.view_cache_active() {
            return;
        }

        let targets: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, n)| {
                (n.view_cache.enabled
                    && n.view_cache.contained_layout
                    && !n.view_cache.layout_definite
                    && n.bounds.size == Size::default()
                    && (n.invalidation.layout || n.invalidation.hit_test))
                    .then_some(id)
            })
            .collect();

        for root in targets {
            self.mark_invalidation_with_source(
                root,
                Invalidation::HitTest,
                UiDebugInvalidationSource::Other,
            );
        }
    }

    fn collapse_observation_index_to_view_cache_roots(
        &self,
        mut index: ObservationIndex,
    ) -> ObservationIndex {
        let mut per_root: HashMap<NodeId, HashMap<ModelId, ObservationMask>> = HashMap::new();
        for (node, entries) in index.by_node.drain() {
            let target = self.nearest_view_cache_root(node).unwrap_or(node);
            let models = per_root.entry(target).or_default();
            for (model, mask) in entries {
                models
                    .entry(model)
                    .and_modify(|m| *m = m.union(mask))
                    .or_insert(mask);
            }
        }

        let mut out = ObservationIndex::default();
        for (node, models) in per_root {
            let mut list: Vec<(ModelId, ObservationMask)> = Vec::with_capacity(models.len());
            for (model, mask) in models {
                list.push((model, mask));
            }
            out.by_node.insert(node, list.clone());
            for (model, mask) in list {
                out.by_model.entry(model).or_default().insert(node, mask);
            }
        }
        out
    }

    fn collapse_global_observation_index_to_view_cache_roots(
        &self,
        mut index: GlobalObservationIndex,
    ) -> GlobalObservationIndex {
        let mut per_root: HashMap<NodeId, HashMap<TypeId, ObservationMask>> = HashMap::new();
        for (node, entries) in index.by_node.drain() {
            let target = self.nearest_view_cache_root(node).unwrap_or(node);
            let globals = per_root.entry(target).or_default();
            for (global, mask) in entries {
                globals
                    .entry(global)
                    .and_modify(|m| *m = m.union(mask))
                    .or_insert(mask);
            }
        }

        let mut out = GlobalObservationIndex::default();
        for (node, globals) in per_root {
            let mut list: Vec<(TypeId, ObservationMask)> = Vec::with_capacity(globals.len());
            for (global, mask) in globals {
                list.push((global, mask));
            }
            out.by_node.insert(node, list.clone());
            for (global, mask) in list {
                out.by_global.entry(global).or_default().insert(node, mask);
            }
        }
        out
    }

    pub(in crate::tree) fn collapse_layout_observations_to_view_cache_roots_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let observed_in_layout = std::mem::take(&mut self.observed_in_layout);
        self.observed_in_layout =
            self.collapse_observation_index_to_view_cache_roots(observed_in_layout);

        let observed_globals_in_layout = std::mem::take(&mut self.observed_globals_in_layout);
        self.observed_globals_in_layout =
            self.collapse_global_observation_index_to_view_cache_roots(observed_globals_in_layout);
    }

    pub(in crate::tree) fn collapse_paint_observations_to_view_cache_roots_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let observed_in_paint = std::mem::take(&mut self.observed_in_paint);
        self.observed_in_paint =
            self.collapse_observation_index_to_view_cache_roots(observed_in_paint);

        let observed_globals_in_paint = std::mem::take(&mut self.observed_globals_in_paint);
        self.observed_globals_in_paint =
            self.collapse_global_observation_index_to_view_cache_roots(observed_globals_in_paint);
    }

    pub(in crate::tree) fn expand_view_cache_layout_invalidations_if_needed(&mut self) {
        if !self.view_cache_active() {
            return;
        }
        let targets: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, n)| (n.view_cache.enabled && n.invalidation.layout).then_some(id))
            .collect();
        if targets.is_empty() {
            return;
        }
        for root in targets {
            self.mark_view_cache_layout_dirty_subtree(root);
        }
    }

    fn mark_view_cache_layout_dirty_subtree(&mut self, root: NodeId) {
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(id) = stack.pop() {
            let (prev, next, layout_before, layout_after) = {
                let Some(n) = self.nodes.get_mut(id) else {
                    continue;
                };
                let prev = n.invalidation;
                let layout_before = n.invalidation.layout;
                n.invalidation.mark(Invalidation::Layout);
                let next = n.invalidation;
                let layout_after = n.invalidation.layout;
                for &child in &n.children {
                    stack.push(child);
                }
                (prev, next, layout_before, layout_after)
            };
            record_layout_invalidation_transition(
                &mut self.layout_invalidations_count,
                layout_before,
                layout_after,
            );
            self.update_invalidation_counters(prev, next);
        }

        self.rebuild_subtree_layout_dirty_counts_and_propagate(root);
    }
}
