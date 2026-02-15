use super::*;

impl<H: UiHost> UiTree<H> {
    pub fn cleanup_subtree(&mut self, services: &mut dyn UiServices, root: NodeId) {
        // Avoid recursion: deep trees can overflow the stack during cleanup.
        let mut stack: Vec<NodeId> = vec![root];
        while let Some(node) = stack.pop() {
            let Some(n) = self.nodes.get(node) else {
                continue;
            };
            let children = n.children.clone();
            for child in children {
                stack.push(child);
            }

            self.cleanup_node_resources(services, node);
        }
    }

    pub(crate) fn set_node_text_boundary_mode_override(
        &mut self,
        node: NodeId,
        mode: Option<fret_runtime::TextBoundaryMode>,
    ) {
        if let Some(n) = self.nodes.get_mut(node) {
            n.text_boundary_mode_override = mode;
        }
    }

    pub(in crate::tree) fn focus_text_boundary_mode_override(
        &self,
    ) -> Option<fret_runtime::TextBoundaryMode> {
        let focus = self.focus?;
        self.nodes
            .get(focus)
            .and_then(|n| n.text_boundary_mode_override)
    }

    pub(in crate::tree) fn cleanup_node_resources(
        &mut self,
        services: &mut dyn UiServices,
        node: NodeId,
    ) {
        let widget = self.nodes.get_mut(node).and_then(|n| n.widget.take());
        if let Some(mut widget) = widget {
            widget.cleanup_resources(services);
            if let Some(n) = self.nodes.get_mut(node) {
                n.widget = Some(widget);
            } else {
                self.deferred_cleanup.push(widget);
            }
        }
    }

    #[track_caller]
    pub(in crate::tree) fn with_widget_mut<R: Default>(
        &mut self,
        node: NodeId,
        f: impl FnOnce(&mut dyn Widget<H>, &mut UiTree<H>) -> R,
    ) -> R {
        fn warn_with_widget_mut_failure_once(node: NodeId, reason: &'static str) {
            static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

            let caller = Location::caller();
            let key = format!(
                "{reason}:{node:?}:{}:{}:{}",
                caller.file(),
                caller.line(),
                caller.column()
            );

            let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
            let first = match seen.lock() {
                Ok(mut guard) => guard.insert(key),
                Err(_) => true,
            };

            if first {
                tracing::error!(
                    ?node,
                    reason,
                    file = caller.file(),
                    line = caller.line(),
                    column = caller.column(),
                    "UiTree widget access failed; returning default"
                );
            }
        }

        let Some(n) = self.nodes.get_mut(node) else {
            if crate::strict_runtime::strict_runtime_enabled() {
                let caller = Location::caller();
                panic!(
                    "UiTree::with_widget_mut: node missing: {node:?} at {}:{}:{}",
                    caller.file(),
                    caller.line(),
                    caller.column()
                );
            }

            warn_with_widget_mut_failure_once(node, "node_missing");
            return R::default();
        };

        let Some(widget) = n.widget.take() else {
            if crate::strict_runtime::strict_runtime_enabled() {
                let caller = Location::caller();
                panic!(
                    "UiTree::with_widget_mut: widget missing (re-entrant borrow?): {node:?} at {}:{}:{}",
                    caller.file(),
                    caller.line(),
                    caller.column()
                );
            }

            warn_with_widget_mut_failure_once(node, "widget_missing");
            return R::default();
        };

        let mut widget = widget;
        let result = catch_unwind(AssertUnwindSafe(|| f(widget.as_mut(), self)));

        if let Some(n) = self.nodes.get_mut(node) {
            n.widget = Some(widget);
        } else {
            self.deferred_cleanup.push(widget);
        }

        match result {
            Ok(result) => result,
            Err(payload) => resume_unwind(payload),
        }
    }

    pub(crate) fn sync_interactivity_gate_widget(
        &mut self,
        node: NodeId,
        present: bool,
        interactive: bool,
    ) {
        if self
            .nodes
            .get(node)
            .and_then(|n| n.widget.as_ref())
            .is_none()
        {
            return;
        }
        #[cfg(debug_assertions)]
        if crate::runtime_config::ui_runtime_config().debug_interactivity_gate_sync {
            eprintln!(
                "sync_interactivity_gate_widget: node={node:?} present={present} interactive={interactive}"
            );
        }
        self.with_widget_mut(node, |w, _ui| {
            w.sync_interactivity_gate(present, interactive);
        });
    }

    pub(crate) fn sync_hit_test_gate_widget(&mut self, node: NodeId, hit_test: bool) {
        if self
            .nodes
            .get(node)
            .and_then(|n| n.widget.as_ref())
            .is_none()
        {
            return;
        }
        #[cfg(debug_assertions)]
        if crate::runtime_config::ui_runtime_config().debug_hit_test_gate_sync {
            eprintln!("sync_hit_test_gate_widget: node={node:?} hit_test={hit_test}");
        }
        self.with_widget_mut(node, |w, _ui| {
            w.sync_hit_test_gate(hit_test);
        });
    }

    pub(crate) fn sync_focus_traversal_gate_widget(&mut self, node: NodeId, traverse: bool) {
        if self
            .nodes
            .get(node)
            .and_then(|n| n.widget.as_ref())
            .is_none()
        {
            return;
        }
        #[cfg(debug_assertions)]
        if crate::runtime_config::ui_runtime_config().debug_focus_traversal_gate_sync {
            eprintln!("sync_focus_traversal_gate_widget: node={node:?} traverse={traverse}");
        }
        self.with_widget_mut(node, |w, _ui| {
            w.sync_focus_traversal_gate(traverse);
        });
    }

    pub(in crate::tree) fn node_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.render_transform(n.bounds)?;
        t.inverse().is_some().then_some(t)
    }

    pub(crate) fn node_children_render_transform(&self, node: NodeId) -> Option<Transform2D> {
        let n = self.nodes.get(node)?;
        let w = n.widget.as_ref()?;
        let t = w.children_render_transform(n.bounds)?;
        t.inverse().is_some().then_some(t)
    }

    pub(in crate::tree) fn apply_vector(t: Transform2D, v: Point) -> Point {
        Point::new(Px(t.a * v.x.0 + t.c * v.y.0), Px(t.b * v.x.0 + t.d * v.y.0))
    }

    pub(crate) fn map_window_point_to_node_layout_space(
        &self,
        target: NodeId,
        window_pos: Point,
    ) -> Option<Point> {
        self.map_window_point_and_vector_to_node_layout_space(target, window_pos, None)
            .map(|(p, _)| p)
    }

    pub(crate) fn map_window_vector_to_node_layout_space(
        &self,
        target: NodeId,
        window_vec: Point,
    ) -> Option<Point> {
        self.map_window_point_and_vector_to_node_layout_space(
            target,
            Point::new(Px(0.0), Px(0.0)),
            Some(window_vec),
        )
        .map(|(_, v)| v.unwrap_or(window_vec))
    }

    fn map_window_point_and_vector_to_node_layout_space(
        &self,
        target: NodeId,
        mut mapped_pos: Point,
        mut mapped_vec: Option<Point>,
    ) -> Option<(Point, Option<Point>)> {
        // Build the chain from target -> root, then walk root -> target.
        let mut chain: Vec<NodeId> = Vec::new();
        let mut cur = Some(target);
        while let Some(id) = cur {
            chain.push(id);
            cur = self.nodes.get(id).and_then(|n| n.parent);
        }
        if chain.is_empty() {
            return None;
        }
        chain.reverse();

        for (idx, &node) in chain.iter().enumerate() {
            let is_target = idx == chain.len().saturating_sub(1);

            let prepaint = self
                .nodes
                .get(node)
                .and_then(|n| {
                    (!self.inspection_active && !n.invalidation.hit_test)
                        .then_some(n.prepaint_hit_test)
                })
                .flatten();

            if let Some(inv) = prepaint
                .and_then(|p| p.render_transform_inv)
                .or_else(|| self.node_render_transform(node).and_then(|t| t.inverse()))
            {
                mapped_pos = inv.apply_point(mapped_pos);
                if let Some(v) = mapped_vec {
                    mapped_vec = Some(Self::apply_vector(inv, v));
                }
            }

            if is_target {
                break;
            }

            let prepaint = self
                .nodes
                .get(node)
                .and_then(|n| {
                    (!self.inspection_active && !n.invalidation.hit_test)
                        .then_some(n.prepaint_hit_test)
                })
                .flatten();
            if let Some(inv) = prepaint
                .and_then(|p| p.children_render_transform_inv)
                .or_else(|| {
                    self.node_children_render_transform(node)
                        .and_then(|t| t.inverse())
                })
            {
                mapped_pos = inv.apply_point(mapped_pos);
                if let Some(v) = mapped_vec {
                    mapped_vec = Some(Self::apply_vector(inv, v));
                }
            }
        }

        Some((mapped_pos, mapped_vec))
    }

    pub(in crate::tree) fn point_in_rounded_rect(
        bounds: Rect,
        radii: Corners,
        position: Point,
    ) -> bool {
        if !bounds.contains(position) {
            return false;
        }

        let w = bounds.size.width.0.max(0.0);
        let h = bounds.size.height.0.max(0.0);
        let limit = 0.5 * w.min(h);

        let tl = Px(radii.top_left.0.max(0.0).min(limit));
        let tr = Px(radii.top_right.0.max(0.0).min(limit));
        let br = Px(radii.bottom_right.0.max(0.0).min(limit));
        let bl = Px(radii.bottom_left.0.max(0.0).min(limit));

        let left = bounds.origin.x.0;
        let top = bounds.origin.y.0;
        let right = left + w;
        let bottom = top + h;

        let x = position.x.0;
        let y = position.y.0;

        // Top-left corner
        if tl.0 > 0.0 && x < left + tl.0 && y < top + tl.0 {
            let cx = left + tl.0;
            let cy = top + tl.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= tl.0 * tl.0;
        }

        // Top-right corner
        if tr.0 > 0.0 && x > right - tr.0 && y < top + tr.0 {
            let cx = right - tr.0;
            let cy = top + tr.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= tr.0 * tr.0;
        }

        // Bottom-right corner
        if br.0 > 0.0 && x > right - br.0 && y > bottom - br.0 {
            let cx = right - br.0;
            let cy = bottom - br.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= br.0 * br.0;
        }

        // Bottom-left corner
        if bl.0 > 0.0 && x < left + bl.0 && y > bottom - bl.0 {
            let cx = left + bl.0;
            let cy = bottom - bl.0;
            let dx = x - cx;
            let dy = y - cy;
            return dx * dx + dy * dy <= bl.0 * bl.0;
        }

        true
    }
}
