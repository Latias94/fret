use super::*;

impl<H: UiHost> UiTree<H> {
    pub(super) fn hit_test(&self, root: NodeId, position: Point) -> Option<NodeId> {
        self.hit_test_node(root, position)
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
}
