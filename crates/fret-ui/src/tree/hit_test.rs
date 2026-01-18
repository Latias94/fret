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

        if let Some(cache) = self.hit_test_path_cache.as_ref()
            && !cache.path.is_empty()
        {
            for &root in layers {
                if root == cache.layer_root {
                    if cache.path.first().copied() == Some(root)
                        && let Some(hit) =
                            self.try_hit_test_along_cached_path(&cache.path, position)
                    {
                        return Some(hit);
                    }

                    let hit = self.hit_test(root, position);
                    self.update_hit_test_path_cache(root, hit);
                    return hit;
                }

                if let Some(hit) = self.hit_test(root, position) {
                    self.update_hit_test_path_cache(root, Some(hit));
                    return Some(hit);
                }
            }

            self.hit_test_path_cache = None;
            return None;
        }

        for &root in layers {
            if let Some(hit) = self.hit_test(root, position) {
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
        let n = self.nodes.get(node)?;
        let widget = n.widget.as_ref();
        let position = if let Some(w) = widget
            && let Some(t) = w.render_transform(n.bounds)
            && let Some(inv) = t.inverse()
        {
            inv.apply_point(position)
        } else {
            position
        };
        let clips_hit_test = widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true);
        if clips_hit_test {
            if !n.bounds.contains(position) {
                return None;
            }
            if let Some(w) = widget
                && let Some(radii) = w.clip_hit_test_corner_radii(n.bounds)
                && !Self::point_in_rounded_rect(n.bounds, radii, position)
            {
                return None;
            }
        }

        let hit_test_children = n
            .widget
            .as_ref()
            .map(|w| w.hit_test_children(n.bounds, position))
            .unwrap_or(true);
        if hit_test_children {
            let child_position = if let Some(w) = widget
                && let Some(t) = w.children_render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position)
            } else {
                position
            };
            for &child in n.children.iter().rev() {
                if let Some(hit) = self.hit_test_node(child, child_position) {
                    return Some(hit);
                }
            }
        }

        let hit = n.bounds.contains(position)
            && n.widget
                .as_ref()
                .map(|w| w.hit_test(n.bounds, position))
                .unwrap_or(true);
        hit.then_some(node)
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
            let position_local = if let Some(w) = widget
                && let Some(t) = w.render_transform(n.bounds)
                && let Some(inv) = t.inverse()
            {
                inv.apply_point(position)
            } else {
                position
            };

            let clips_hit_test = widget.map(|w| w.clips_hit_test(n.bounds)).unwrap_or(true);
            if clips_hit_test {
                if !n.bounds.contains(position_local) {
                    return None;
                }
                if let Some(w) = widget
                    && let Some(radii) = w.clip_hit_test_corner_radii(n.bounds)
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

            let child_position = if let Some(w) = widget
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
                if let Some(w) = sib.widget.as_ref() {
                    if w.render_transform(sib.bounds).is_some() || !w.clips_hit_test(sib.bounds) {
                        return None;
                    }
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
