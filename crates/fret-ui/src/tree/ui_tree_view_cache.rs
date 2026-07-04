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
        self.window_paint_replay.ingest_previous_frame_scene(scene);
    }

    pub(in crate::tree) fn view_cache_active(&self) -> bool {
        self.view_cache_enabled && !self.inspection_active
    }

    pub(in crate::tree) fn nearest_view_cache_root(&self, node: NodeId) -> Option<NodeId> {
        if !self.node_is_reachable_from_layer_forest(node) {
            return None;
        }

        let mut current = Some(node);
        while let Some(id) = current {
            let n = self.nodes.get(id)?;
            if n.view_cache.enabled {
                return Some(id);
            }
            current = self.live_parent_in_layer_forest(id);
        }
        None
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
        self.boundary_allows_contained_relayout(node) && n.bounds.size != Size::default()
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
        contain_layout_when_bounds_known: bool,
        layout_definite: bool,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            let next = ViewCacheFlags::from_contain_layout_when_bounds_known(
                enabled,
                contain_layout_when_bounds_known,
                layout_definite,
            );
            if n.view_cache == next {
                return;
            }
            n.view_cache = next;
            self.sync_view_boundary_state_for_node(node);
        }
    }

    pub(crate) fn set_node_view_cache_needs_rerender(&mut self, node: NodeId, needs: bool) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.view_cache_needs_rerender = needs;
        }
        if !needs {
            self.clear_boundary_layout_dirty(node);
        }
    }

    pub(in crate::tree) fn clear_boundary_dirty_tracking_if_clean(&mut self, node: NodeId) {
        let should_clear = self
            .nodes
            .get(node)
            .is_none_or(|n| !n.view_cache_needs_rerender && !n.invalidation.layout);
        if should_clear {
            self.clear_boundary_layout_dirty(node);
        }
    }

    pub(in crate::tree) fn mark_view_cache_roots_needs_rerender_from_snapshot(
        &mut self,
        start: NodeId,
        snapshot: Option<&UiDispatchSnapshot>,
        source: UiDebugInvalidationSource,
        detail: UiDebugInvalidationDetail,
    ) {
        if !self.view_cache_active() {
            return;
        }

        let snapshot =
            snapshot.filter(|snapshot| snapshot.topology_epoch == self.live_topology_epoch());
        let mut current = Some(start);
        while let Some(id) = current {
            let next = match snapshot {
                Some(snapshot) => snapshot.parent.get(id).copied().flatten(),
                None => self.live_parent_in_layer_forest(id),
            };

            if let Some(n) = self.nodes.get_mut(id)
                && n.view_cache.enabled
            {
                n.view_cache_needs_rerender = true;
                self.mark_boundary_layout_dirty(id, source, detail);
            }

            current = next;
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
            let next_parent = self.live_parent_in_layer_forest(id);
            if let Some(n) = self.nodes.get_mut(id)
                && n.view_cache.enabled
            {
                n.view_cache_needs_rerender = true;
                self.mark_boundary_layout_dirty(id, source, detail);
            }
            current = next_parent;
        }
    }

    /// Repair invalidation propagation for newly mounted auto-sized cache roots.
    ///
    /// During declarative mounting we may discover `ViewCache` roots before retained parent
    /// metadata is fully connected. When view caching is active, invalidation propagation can be
    /// truncated at cache roots, and a cache root that is only marked dirty on itself may never be
    /// laid out by its (still-clean) ancestors. This shows up as cache-root subtrees stuck at
    /// `Rect::default()` origins (e.g. scripted clicks using semantics bounds land in the wrong
    /// place).
    ///
    /// Call this after the declarative child graph is mounted and before `layout_all` so the next
    /// layout pass walks far enough to place newly mounted cache-root subtrees.
    pub(crate) fn propagate_auto_sized_view_cache_root_invalidations(&mut self) {
        if !self.view_cache_active() {
            return;
        }

        let targets: Vec<NodeId> = self
            .nodes
            .iter()
            .filter_map(|(id, n)| {
                (n.view_cache.enabled
                    && n.view_cache.layout_contained_when_bounds_known()
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
            if !layout_before && layout_after {
                self.debug_note_layout_dirty_source(
                    id,
                    root,
                    UiDebugInvalidationSource::Other,
                    UiDebugInvalidationDetail::ViewCacheLayoutDirtyExpansion,
                );
            }
            self.update_invalidation_counters(prev, next);
        }

        self.rebuild_subtree_layout_dirty_counts_and_propagate(root);
    }
}
