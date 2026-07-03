use super::*;

#[derive(Debug, Default, Clone)]
pub(super) struct HitTestPathRoutingCacheState {
    /// Window-owned primary pointer routing path cache.
    ///
    /// Secondary hit tests suspend publication so occlusion/hover probes cannot replace the
    /// primary pointer route with a probe-specific layer root.
    entry: Option<HitTestPathCache>,
    suspension_depth: u32,
}

#[derive(Debug, Clone)]
struct HitTestPathCache {
    layer_root: NodeId,
    path: Vec<NodeId>,
}

impl HitTestPathRoutingCacheState {
    fn clear(&mut self) {
        self.entry = None;
    }

    fn begin_suspended_query(&mut self) {
        self.suspension_depth = self.suspension_depth.saturating_add(1);
    }

    fn end_suspended_query(&mut self) {
        self.suspension_depth = self.suspension_depth.saturating_sub(1);
    }

    fn suspended(&self) -> bool {
        self.suspension_depth > 0
    }

    fn take_for_query(&mut self) -> Option<HitTestPathCache> {
        if self.suspended() {
            None
        } else {
            self.entry.take()
        }
    }

    fn restore_for_query(&mut self, entry: HitTestPathCache) {
        if !self.suspended() {
            self.entry = Some(entry);
        }
    }

    fn clear_after_query(&mut self) {
        if !self.suspended() {
            self.clear();
        }
    }

    fn clear_layer_after_query(&mut self, layer_root: NodeId) {
        if !self.suspended()
            && self
                .entry
                .as_ref()
                .is_some_and(|entry| entry.layer_root == layer_root)
        {
            self.clear();
        }
    }

    fn set_after_query(&mut self, layer_root: NodeId, path: Vec<NodeId>) {
        if !self.suspended() {
            self.entry = Some(HitTestPathCache { layer_root, path });
        }
    }

    #[cfg(test)]
    fn has_entry_for_layer(&self, layer_root: NodeId) -> bool {
        self.entry
            .as_ref()
            .is_some_and(|entry| entry.layer_root == layer_root)
    }
}

impl<H: UiHost> UiTree<H> {
    pub(super) fn hit_test(&self, root: NodeId, position: Point) -> Option<NodeId> {
        self.hit_test_node(root, position)
    }

    pub(super) fn clear_hit_test_path_cache(&mut self) {
        self.hit_test_path_cache.clear();
    }

    pub(super) fn begin_suspended_hit_test_path_cache_query(&mut self) {
        self.hit_test_path_cache.begin_suspended_query();
    }

    pub(super) fn end_suspended_hit_test_path_cache_query(&mut self) {
        self.hit_test_path_cache.end_suspended_query();
    }

    #[cfg(test)]
    pub(crate) fn test_hit_test_path_cache_has_entry_for_layer(&self, layer_root: NodeId) -> bool {
        self.hit_test_path_cache.has_entry_for_layer(layer_root)
    }

    pub(super) fn hit_test_layers_cached_with_root(
        &mut self,
        layers: &[NodeId],
        position: Point,
    ) -> Option<(NodeId, NodeId)> {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let layers_len = layers.len() as u64;
        let (hit, elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.hit_test.layers", layers_len),
            || {
                if layers.is_empty() {
                    self.hit_test_path_cache.clear_after_query();
                    return None;
                }

                if let Some(cache) = self.hit_test_path_cache.take_for_query()
                    && !cache.path.is_empty()
                {
                    for &root in layers {
                        if root == cache.layer_root {
                            let bounds_tree_enabled = self.hit_test_bounds_tree_layer_enabled(root);

                            if !bounds_tree_enabled
                                && cache.path.first().copied() == Some(root)
                                && let Some(hit) = {
                                    let path_len = cache.path.len() as u64;
                                    let (hit, elapsed) = fret_perf::measure_span(
                                        self.debug_enabled,
                                        trace_enabled,
                                        || {
                                            tracing::trace_span!(
                                                "fret.ui.hit_test.cached_path",
                                                root = ?root,
                                                path_len,
                                            )
                                        },
                                        || {
                                            self.try_hit_test_along_cached_path(
                                                &cache.path,
                                                position,
                                            )
                                        },
                                    );
                                    if let Some(elapsed) = elapsed {
                                        self.debug_stats.hit_test_cached_path_time += elapsed;
                                    }
                                    hit
                                }
                            {
                                if self.debug_enabled {
                                    self.debug_stats.hit_test_path_cache_hits =
                                        self.debug_stats.hit_test_path_cache_hits.saturating_add(1);
                                }
                                self.hit_test_path_cache.restore_for_query(cache);
                                return Some((root, hit));
                            }

                            if self.debug_enabled && !bounds_tree_enabled {
                                self.debug_stats.hit_test_path_cache_misses = self
                                    .debug_stats
                                    .hit_test_path_cache_misses
                                    .saturating_add(1);
                            }
                            let hit = self.hit_test_layer_bounds_tree_or_fallback(root, position);
                            self.update_hit_test_path_cache(root, hit);
                            return hit.map(|hit| (root, hit));
                        }

                        if let Some(hit) =
                            self.hit_test_layer_bounds_tree_or_fallback(root, position)
                        {
                            self.update_hit_test_path_cache(root, Some(hit));
                            return Some((root, hit));
                        }
                    }

                    self.hit_test_path_cache.clear_after_query();
                    return None;
                }

                for &root in layers {
                    if let Some(hit) = self.hit_test_layer_bounds_tree_or_fallback(root, position) {
                        self.update_hit_test_path_cache(root, Some(hit));
                        return Some((root, hit));
                    }
                }

                self.hit_test_path_cache.clear_after_query();
                None
            },
        );
        if self.debug_enabled {
            self.debug_stats.hit_test_queries = self.debug_stats.hit_test_queries.saturating_add(1);
            if let Some(elapsed) = elapsed {
                self.debug_stats.hit_test_time += elapsed;
            }
        }
        hit
    }

    pub(super) fn hit_test_layers_cached(
        &mut self,
        layers: &[NodeId],
        position: Point,
    ) -> Option<NodeId> {
        self.hit_test_layers_cached_with_root(layers, position)
            .map(|(_root, hit)| hit)
    }

    pub(super) fn hit_test_layers_with_root(
        &self,
        layers: &[NodeId],
        position: Point,
    ) -> Option<(NodeId, NodeId)> {
        for &root in layers {
            if let Some(hit) = self.hit_test(root, position) {
                return Some((root, hit));
            }
        }
        None
    }

    pub(super) fn hit_test_layers(&self, layers: &[NodeId], position: Point) -> Option<NodeId> {
        self.hit_test_layers_with_root(layers, position)
            .map(|(_root, hit)| hit)
    }

    fn hit_test_node(&self, node: NodeId, position: Point) -> Option<NodeId> {
        // Avoid recursion: deep UI trees can overflow the stack during hit testing.
        enum Frame {
            Visit {
                node: NodeId,
                position: Point,
                /// When true, treat this node as hit-test clipped by its bounds even if the node
                /// itself does not clip hit testing.
                force_clip_to_bounds: bool,
            },
            SelfCheck(NodeId, Point),
        }

        let mut stack: Vec<Frame> = Vec::new();
        stack.push(Frame::Visit {
            node,
            position,
            force_clip_to_bounds: false,
        });

        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Visit {
                    node,
                    position,
                    force_clip_to_bounds,
                } => {
                    let Some(n) = self.nodes.get(node) else {
                        continue;
                    };
                    let widget = n.widget.as_ref();

                    let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
                        .then_some(n.prepaint_hit_test)
                        .flatten();
                    let render_transform_inv =
                        prepaint.as_ref().and_then(|p| p.render_transform_inv);
                    let children_render_transform_inv = prepaint
                        .as_ref()
                        .and_then(|p| p.children_render_transform_inv);
                    let clips_hit_test = prepaint
                        .as_ref()
                        .map(|p| p.clips_hit_test)
                        .unwrap_or_else(|| {
                            widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true)
                        });
                    let corner_radii = prepaint
                        .as_ref()
                        .and_then(|p| p.clip_hit_test_corner_radii)
                        .or_else(|| widget.and_then(|w| w.clip_hit_test_corner_radii(n.bounds)));

                    let position_local = if let Some(inv) = render_transform_inv {
                        inv.apply_point(position)
                    } else if let Some(w) = widget
                        && let Some(t) = w.render_transform(n.bounds)
                        && let Some(inv) = t.inverse()
                    {
                        inv.apply_point(position)
                    } else {
                        position
                    };

                    if clips_hit_test || force_clip_to_bounds {
                        if !n.bounds.contains(position_local) {
                            continue;
                        }
                        if let Some(radii) = corner_radii
                            && !Self::point_in_rounded_rect(n.bounds, radii, position_local)
                        {
                            continue;
                        }
                    }

                    let hit_test_children = widget
                        .map(|w| w.hit_test_children(n.bounds, position_local))
                        .unwrap_or(true);
                    if hit_test_children && !n.children.is_empty() {
                        let child_position = if let Some(inv) = children_render_transform_inv {
                            inv.apply_point(position_local)
                        } else if let Some(w) = widget
                            && let Some(t) = w.children_render_transform(n.bounds)
                            && let Some(inv) = t.inverse()
                        {
                            inv.apply_point(position_local)
                        } else {
                            position_local
                        };

                        // Children should be hit-tested before the node itself.
                        stack.push(Frame::SelfCheck(node, position_local));
                        for &child in n.children.iter() {
                            stack.push(Frame::Visit {
                                node: child,
                                position: child_position,
                                force_clip_to_bounds,
                            });
                        }
                        continue;
                    }

                    let hit = n.bounds.contains(position_local)
                        && widget
                            .map(|w| w.hit_test(n.bounds, position_local))
                            .unwrap_or(true);
                    if hit {
                        return Some(node);
                    }
                }
                Frame::SelfCheck(node, position_local) => {
                    let Some(n) = self.nodes.get(node) else {
                        continue;
                    };
                    let widget = n.widget.as_ref();
                    let hit = n.bounds.contains(position_local)
                        && widget
                            .map(|w| w.hit_test(n.bounds, position_local))
                            .unwrap_or(true);
                    if hit {
                        return Some(node);
                    }
                }
            }
        }

        None
    }

    fn hit_test_layer_bounds_tree_or_fallback(
        &mut self,
        root: NodeId,
        position: Point,
    ) -> Option<NodeId> {
        let trace_enabled = tracing::enabled!(tracing::Level::TRACE);
        let ((query, query_stats), elapsed) = fret_perf::measure_span(
            self.debug_enabled,
            trace_enabled,
            || tracing::trace_span!("fret.ui.hit_test.bounds_tree_query", root = ?root),
            || self.query_hit_test_bounds_tree(root, position, self.debug_enabled),
        );
        if let Some(elapsed) = elapsed {
            self.debug_stats.hit_test_bounds_tree_query_time += elapsed;
        }
        if self.debug_enabled {
            self.debug_stats.hit_test_bounds_tree_queries = self
                .debug_stats
                .hit_test_bounds_tree_queries
                .saturating_add(1);
            self.debug_stats.hit_test_bounds_tree_nodes_visited = self
                .debug_stats
                .hit_test_bounds_tree_nodes_visited
                .saturating_add(query_stats.nodes_visited);
            self.debug_stats.hit_test_bounds_tree_nodes_pushed = self
                .debug_stats
                .hit_test_bounds_tree_nodes_pushed
                .saturating_add(query_stats.nodes_pushed);
            match query {
                super::bounds_tree::HitTestBoundsTreeQuery::Disabled => {
                    self.debug_stats.hit_test_bounds_tree_disabled = self
                        .debug_stats
                        .hit_test_bounds_tree_disabled
                        .saturating_add(1);
                }
                super::bounds_tree::HitTestBoundsTreeQuery::Miss => {
                    self.debug_stats.hit_test_bounds_tree_misses = self
                        .debug_stats
                        .hit_test_bounds_tree_misses
                        .saturating_add(1);
                }
                super::bounds_tree::HitTestBoundsTreeQuery::Hit(_) => {
                    self.debug_stats.hit_test_bounds_tree_hits =
                        self.debug_stats.hit_test_bounds_tree_hits.saturating_add(1);
                }
            }
        }

        match query {
            super::bounds_tree::HitTestBoundsTreeQuery::Disabled => {
                let (hit, elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || tracing::trace_span!("fret.ui.hit_test.fallback_traversal", root = ?root),
                    || self.hit_test(root, position),
                );
                if let Some(elapsed) = elapsed {
                    self.debug_stats.hit_test_fallback_traversal_time += elapsed;
                }
                hit
            }
            super::bounds_tree::HitTestBoundsTreeQuery::Miss => None,
            super::bounds_tree::HitTestBoundsTreeQuery::Hit(candidate) => {
                let (accepted, elapsed) = fret_perf::measure_span(
                    self.debug_enabled,
                    trace_enabled,
                    || {
                        tracing::trace_span!(
                            "fret.ui.hit_test.candidate_self_only",
                            root = ?root,
                            candidate = ?candidate,
                        )
                    },
                    || {
                        self.hit_test_node_self_only(candidate, position)
                            && self
                                .hit_test_candidate_reachable_from_root(root, candidate, position)
                    },
                );
                if let Some(elapsed) = elapsed {
                    self.debug_stats.hit_test_candidate_self_only_time += elapsed;
                }
                if accepted {
                    Some(candidate)
                } else {
                    if self.debug_enabled {
                        self.debug_stats.hit_test_bounds_tree_candidate_rejected = self
                            .debug_stats
                            .hit_test_bounds_tree_candidate_rejected
                            .saturating_add(1);
                    }
                    let (hit, elapsed) = fret_perf::measure_span(
                        self.debug_enabled,
                        trace_enabled,
                        || {
                            tracing::trace_span!(
                                "fret.ui.hit_test.fallback_traversal",
                                root = ?root,
                                rejected_candidate = ?candidate,
                            )
                        },
                        || self.hit_test(root, position),
                    );
                    if let Some(elapsed) = elapsed {
                        self.debug_stats.hit_test_fallback_traversal_time += elapsed;
                    }
                    hit
                }
            }
        }
    }

    fn hit_test_candidate_reachable_from_root(
        &self,
        root: NodeId,
        candidate: NodeId,
        position: Point,
    ) -> bool {
        // The bounds-tree fast path can return a deep descendant without proving that all
        // ancestors would allow hit-test traversal (e.g. `HitTestGate(hit_test=false)`).
        // Validate that the root->candidate chain is hit-test traversable in the current
        // coordinate space.

        let Some(path) = self.path_from_root_to_node_via_children(root, candidate) else {
            return false;
        };

        let mut position = position;
        for (idx, &node) in path.iter().enumerate() {
            let Some(n) = self.nodes.get(node) else {
                return false;
            };
            let widget = n.widget.as_ref();

            let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
                .then_some(n.prepaint_hit_test)
                .flatten();
            let render_transform_inv = prepaint.as_ref().and_then(|p| p.render_transform_inv);
            let children_render_transform_inv = prepaint
                .as_ref()
                .and_then(|p| p.children_render_transform_inv);
            let clips_hit_test = prepaint
                .as_ref()
                .map(|p| p.clips_hit_test)
                .unwrap_or_else(|| widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true));
            let corner_radii = prepaint
                .as_ref()
                .and_then(|p| p.clip_hit_test_corner_radii)
                .or_else(|| widget.and_then(|w| w.clip_hit_test_corner_radii(n.bounds)));

            let position_local = if let Some(inv) = render_transform_inv {
                inv.apply_point(position)
            } else if let Some(w) = widget
                && let Some(t) = w.render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position)
            } else {
                position
            };

            if clips_hit_test {
                if !n.bounds.contains(position_local) {
                    return false;
                }
                if let Some(radii) = corner_radii
                    && !Self::point_in_rounded_rect(n.bounds, radii, position_local)
                {
                    return false;
                }
            }

            let Some(next) = path.get(idx + 1).copied() else {
                return true;
            };

            let hit_test_children = widget
                .map(|w| w.hit_test_children(n.bounds, position_local))
                .unwrap_or(true);
            if !hit_test_children {
                return false;
            }

            let child_position = if let Some(inv) = children_render_transform_inv {
                inv.apply_point(position_local)
            } else if let Some(w) = widget
                && let Some(t) = w.children_render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position_local)
            } else {
                position_local
            };

            if !n.children.contains(&next) {
                return false;
            }

            position = child_position;
        }

        false
    }

    fn path_from_root_to_node_via_children(
        &self,
        root: NodeId,
        target: NodeId,
    ) -> Option<Vec<NodeId>> {
        if !self.nodes.contains_key(root) || !self.nodes.contains_key(target) {
            return None;
        }

        let mut path: Vec<NodeId> = Vec::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut stack: Vec<(NodeId, bool)> = vec![(root, false)];

        while let Some((node, exiting)) = stack.pop() {
            if exiting {
                path.pop();
                continue;
            }

            let Some(record) = self.nodes.get(node) else {
                continue;
            };
            if !visited.insert(node) {
                continue;
            }

            path.push(node);
            if node == target {
                return Some(path);
            }

            stack.push((node, true));
            for &child in record.children.iter().rev() {
                stack.push((child, false));
            }
        }

        None
    }

    fn hit_test_node_self_only(&self, node: NodeId, position: Point) -> bool {
        let Some(n) = self.nodes.get(node) else {
            return false;
        };
        let widget = n.widget.as_ref();

        let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
            .then_some(n.prepaint_hit_test)
            .flatten();
        let render_transform_inv = prepaint.as_ref().and_then(|p| p.render_transform_inv);
        let clips_hit_test = prepaint
            .as_ref()
            .map(|p| p.clips_hit_test)
            .unwrap_or_else(|| widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true));
        let corner_radii = prepaint
            .as_ref()
            .and_then(|p| p.clip_hit_test_corner_radii)
            .or_else(|| widget.and_then(|w| w.clip_hit_test_corner_radii(n.bounds)));

        let position_local = if let Some(inv) = render_transform_inv {
            inv.apply_point(position)
        } else if let Some(w) = widget
            && let Some(t) = w.render_transform(n.bounds)
            && let Some(inv) = t.inverse()
        {
            inv.apply_point(position)
        } else {
            position
        };

        if clips_hit_test {
            if !n.bounds.contains(position_local) {
                return false;
            }
            if let Some(radii) = corner_radii
                && !Self::point_in_rounded_rect(n.bounds, radii, position_local)
            {
                return false;
            }
        }

        n.bounds.contains(position_local)
            && widget
                .map(|w| w.hit_test(n.bounds, position_local))
                .unwrap_or(true)
    }

    fn update_hit_test_path_cache(&mut self, layer_root: NodeId, hit: Option<NodeId>) {
        let Some(hit) = hit else {
            self.hit_test_path_cache.clear_layer_after_query(layer_root);
            return;
        };

        let Some(path) = self.path_from_root_to_node_via_children(layer_root, hit) else {
            self.hit_test_path_cache.clear_after_query();
            return;
        };
        self.hit_test_path_cache.set_after_query(layer_root, path);
    }

    fn try_hit_test_along_cached_path(&self, path: &[NodeId], position: Point) -> Option<NodeId> {
        let mut position = position;
        let force_clip_to_bounds = false;

        for (idx, &node) in path.iter().enumerate() {
            let n = self.nodes.get(node)?;
            let widget = n.widget.as_ref();

            let prepaint = (!self.inspection_active && !n.invalidation.hit_test)
                .then_some(n.prepaint_hit_test)
                .flatten();
            let render_transform_inv = prepaint.as_ref().and_then(|p| p.render_transform_inv);
            let children_render_transform_inv = prepaint
                .as_ref()
                .and_then(|p| p.children_render_transform_inv);
            let clips_hit_test = prepaint
                .as_ref()
                .map(|p| p.clips_hit_test)
                .unwrap_or_else(|| widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true));
            let corner_radii = prepaint
                .as_ref()
                .and_then(|p| p.clip_hit_test_corner_radii)
                .or_else(|| widget.and_then(|w| w.clip_hit_test_corner_radii(n.bounds)));

            let position_local = if let Some(inv) = render_transform_inv {
                inv.apply_point(position)
            } else if let Some(w) = widget
                && let Some(t) = w.render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position)
            } else {
                position
            };
            if clips_hit_test || force_clip_to_bounds {
                if !n.bounds.contains(position_local) {
                    return None;
                }
                if let Some(radii) = corner_radii
                    && !Self::point_in_rounded_rect(n.bounds, radii, position_local)
                {
                    return None;
                }
            }

            let next = path.get(idx + 1).copied();
            let Some(next) = next else {
                // The cached path ends here. If this node can hit-test children, then a different
                // descendant could become the correct hit for a different pointer position. In
                // that case, fall back to the full hit-test implementation.
                let hit_test_children = widget
                    .map(|w| w.hit_test_children(n.bounds, position_local))
                    .unwrap_or(true);
                if hit_test_children && !n.children.is_empty() {
                    return None;
                }

                let hit = n.bounds.contains(position_local)
                    && widget
                        .map(|w| w.hit_test(n.bounds, position_local))
                        .unwrap_or(true);
                return hit.then_some(node);
            };

            let hit_test_children = widget
                .map(|w| w.hit_test_children(n.bounds, position_local))
                .unwrap_or(true);
            if !hit_test_children {
                return None;
            }

            let child_position = if let Some(inv) = children_render_transform_inv {
                inv.apply_point(position_local)
            } else if let Some(w) = widget
                && let Some(t) = w.children_render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position_local)
            } else {
                position_local
            };

            // Safety: ensure no higher-z siblings could intercept the hit before the cached child.
            let mut found = false;
            for &child in n.children.iter().rev() {
                if child == next {
                    found = true;
                    break;
                }
                if !self.nodes.contains_key(child) {
                    continue;
                }

                if self.hit_test_node(child, child_position).is_some() {
                    return None;
                }
            }
            if !found {
                return None;
            }

            position = child_position;
        }

        None
    }
}
