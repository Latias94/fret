use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn hit_test(&self, root: NodeId, position: Point) -> Option<NodeId> {
        self.hit_test_node(root, position)
    }

    pub(super) fn hit_test_layers_cached(
        &mut self,
        layers: &[NodeId],
        position: Point,
    ) -> Option<NodeId> {
        if layers.is_empty() {
            self.hit_test_path_cache = None;
            return None;
        }

        if let Some(cache) = self.hit_test_path_cache.take()
            && !cache.path.is_empty()
        {
            for &root in layers {
                if root == cache.layer_root {
                    let bounds_tree_enabled = self.hit_test_bounds_trees.layer_enabled(root);

                    if !bounds_tree_enabled
                        && cache.path.first().copied() == Some(root)
                        && let Some(hit) = {
                            let started = self.debug_enabled.then(std::time::Instant::now);
                            let hit = self.try_hit_test_along_cached_path(&cache.path, position);
                            if let Some(started) = started {
                                self.debug_stats.hit_test_cached_path_time += started.elapsed();
                            }
                            hit
                        }
                    {
                        if self.debug_enabled {
                            self.debug_stats.hit_test_path_cache_hits =
                                self.debug_stats.hit_test_path_cache_hits.saturating_add(1);
                        }
                        self.hit_test_path_cache = Some(cache);
                        return Some(hit);
                    }

                    if self.debug_enabled && !bounds_tree_enabled {
                        self.debug_stats.hit_test_path_cache_misses = self
                            .debug_stats
                            .hit_test_path_cache_misses
                            .saturating_add(1);
                    }
                    let hit = self.hit_test_layer_bounds_tree_or_fallback(root, position);
                    self.update_hit_test_path_cache(root, hit);
                    return hit;
                }

                if let Some(hit) = self.hit_test_layer_bounds_tree_or_fallback(root, position) {
                    self.update_hit_test_path_cache(root, Some(hit));
                    return Some(hit);
                }
            }

            self.hit_test_path_cache = None;
            return None;
        }

        for &root in layers {
            if let Some(hit) = self.hit_test_layer_bounds_tree_or_fallback(root, position) {
                self.update_hit_test_path_cache(root, Some(hit));
                return Some(hit);
            }
        }

        self.hit_test_path_cache = None;
        None
    }

    pub(super) fn hit_test_layers(&self, layers: &[NodeId], position: Point) -> Option<NodeId> {
        for &root in layers {
            if let Some(hit) = self.hit_test(root, position) {
                return Some(hit);
            }
        }
        None
    }

    fn hit_test_node(&self, node: NodeId, position: Point) -> Option<NodeId> {
        // Avoid recursion: deep UI trees can overflow the stack during hit testing.
        enum Frame {
            Visit(NodeId, Point),
            SelfCheck(NodeId, Point),
        }

        let mut stack: Vec<Frame> = Vec::new();
        stack.push(Frame::Visit(node, position));

        while let Some(frame) = stack.pop() {
            match frame {
                Frame::Visit(node, position) => {
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

                    if clips_hit_test {
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
                            stack.push(Frame::Visit(child, child_position));
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
        let started = self.debug_enabled.then(std::time::Instant::now);
        let (query, query_stats) =
            self.hit_test_bounds_trees
                .query(root, position, self.debug_enabled);
        if let Some(started) = started {
            self.debug_stats.hit_test_bounds_tree_query_time += started.elapsed();
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
                let started = self.debug_enabled.then(std::time::Instant::now);
                let hit = self.hit_test(root, position);
                if let Some(started) = started {
                    self.debug_stats.hit_test_fallback_traversal_time += started.elapsed();
                }
                hit
            }
            super::bounds_tree::HitTestBoundsTreeQuery::Miss => None,
            super::bounds_tree::HitTestBoundsTreeQuery::Hit(candidate) => {
                let started = self.debug_enabled.then(std::time::Instant::now);
                let accepted = self.hit_test_node_self_only(candidate, position);
                if let Some(started) = started {
                    self.debug_stats.hit_test_candidate_self_only_time += started.elapsed();
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
                    let started = self.debug_enabled.then(std::time::Instant::now);
                    let hit = self.hit_test(root, position);
                    if let Some(started) = started {
                        self.debug_stats.hit_test_fallback_traversal_time += started.elapsed();
                    }
                    hit
                }
            }
        }
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
            if self
                .hit_test_path_cache
                .as_ref()
                .is_some_and(|c| c.layer_root == layer_root)
            {
                self.hit_test_path_cache = None;
            }
            return;
        };

        let mut path_rev: Vec<NodeId> = Vec::new();
        let mut current = Some(hit);
        while let Some(id) = current {
            path_rev.push(id);
            if id == layer_root {
                break;
            }
            current = self.nodes.get(id).and_then(|n| n.parent);
        }
        if path_rev.last().copied() != Some(layer_root) {
            self.hit_test_path_cache = None;
            return;
        }
        path_rev.reverse();
        self.hit_test_path_cache = Some(super::HitTestPathCache {
            layer_root,
            path: path_rev,
        });
    }

    fn try_hit_test_along_cached_path(&self, path: &[NodeId], position: Point) -> Option<NodeId> {
        let mut position = position;

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
            if clips_hit_test {
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
                let Some(sib) = self.nodes.get(child) else {
                    continue;
                };

                // Conservative correctness gate: if a sibling uses transforms or does not clip hit
                // testing to its bounds, we cannot cheaply prove it won't intercept the hit.
                if let Some(p) = (!self.inspection_active && !sib.invalidation.hit_test)
                    .then_some(sib.prepaint_hit_test)
                    .flatten()
                {
                    if p.render_transform_inv.is_some() || !p.clips_hit_test {
                        return None;
                    }
                } else if let Some(w) = sib.widget.as_ref()
                    && (w.render_transform(sib.bounds).is_some() || !w.clips_hit_test(sib.bounds))
                {
                    return None;
                }

                if sib.bounds.contains(child_position) {
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
