use super::*;

impl<H: UiHost> UiTree<H> {
    fn apply_interaction_record(&mut self, record: &InteractionRecord) {
        let (prev, next) = {
            let Some(n) = self.nodes.get_mut(record.node) else {
                return;
            };
            let prev = n.invalidation;
            n.prepaint_hit_test = Some(super::PrepaintHitTestCache {
                render_transform_inv: record.render_transform_inv,
                children_render_transform_inv: record.children_render_transform_inv,
                clips_hit_test: record.clips_hit_test,
                clip_hit_test_corner_radii: record.clip_hit_test_corner_radii,
                is_focusable: record.is_focusable,
                focus_traversal_children: record.focus_traversal_children,
                can_scroll_descendant_into_view: record.can_scroll_descendant_into_view,
            });
            n.invalidation.hit_test = false;
            (prev, n.invalidation)
        };
        self.update_invalidation_counters(prev, next);
    }

    pub(super) fn prepaint_interaction_node(
        &mut self,
        app: &mut H,
        node: NodeId,
        scale_factor: f32,
        theme_revision: u64,
    ) {
        if self.debug_enabled {
            self.debug_stats.prepaint_nodes_visited =
                self.debug_stats.prepaint_nodes_visited.saturating_add(1);
        }

        let (
            bounds,
            invalidation,
            is_view_cache_root,
            prev_cache,
            is_manual_cache_root,
            runs_widget_prepaint,
        ) = match self.nodes.get(node) {
            Some(n) => (
                n.bounds,
                n.invalidation,
                self.view_cache_active() && n.view_cache.enabled,
                n.interaction_cache,
                n.view_cache.enabled && n.element.is_none(),
                // Retained/manual cache roots use the same cache-boundary flag to opt into
                // per-frame prepaint work even when global view-cache reuse is disabled.
                // Their prepaint hooks drive live runtime state (for example chart engine
                // stepping, dock drag routes, or cull-window updates), so gating them behind
                // `view_cache_active()` breaks correctness in the default no-cache mode.
                n.view_cache.enabled && (self.view_cache_active() || n.element.is_none()),
            ),
            None => return,
        };

        let child_transform = self
            .node_children_render_transform(node)
            .unwrap_or(Transform2D::IDENTITY);
        let key = PaintCacheKey::new(
            bounds,
            scale_factor,
            theme_revision,
            crate::tree::paint_style::PaintStyleState::default(),
            None,
            child_transform,
        );

        if is_view_cache_root && is_manual_cache_root {
            let contained_layout = self
                .nodes
                .get(node)
                .map(|n| n.view_cache.contained_layout)
                .unwrap_or(false);
            self.debug_record_view_cache_root(
                node,
                self.should_reuse_view_cache_node(node),
                contained_layout,
                crate::tree::UiDebugCacheRootReuseReason::ManualCacheRoot,
            );
        }

        if runs_widget_prepaint {
            let window = self.window;
            let sf = scale_factor;
            self.begin_prepaint_outputs_for_node(node, key);
            self.with_widget_mut(node, |widget, tree| {
                let mut cx = crate::widget::PrepaintCx {
                    app,
                    tree,
                    node,
                    window,
                    bounds,
                    scale_factor: sf,
                };
                widget.prepaint(&mut cx);
            });
        }

        let can_reuse =
            is_view_cache_root && self.should_reuse_view_cache_node(node) && !invalidation.hit_test;
        if can_reuse
            && let Some(prev) = prev_cache
            && prev.generation == self.interaction_cache.source_generation
            && prev.key == key
        {
            let range = prev.start as usize..prev.end as usize;
            if range.start <= range.end && range.end <= self.interaction_cache.prev_records.len() {
                let start = self.interaction_cache.records.len();
                self.interaction_cache.replay_scratch.clear();
                self.interaction_cache
                    .replay_scratch
                    .extend_from_slice(&self.interaction_cache.prev_records[range]);
                for i in 0..self.interaction_cache.replay_scratch.len() {
                    let mut record = self.interaction_cache.replay_scratch[i];
                    // View-cache reuse can legitimately skip rerender/layout for the subtree, but
                    // hit-test-only invalidations (e.g. scroll handle offset changes) still need
                    // up-to-date interaction transforms so pointer routing stays correct.
                    //
                    // If the node was marked hit-test-invalidated this frame, refresh the
                    // interaction record from live widget state instead of replaying the cached
                    // inverse transforms.
                    if self
                        .nodes
                        .get(record.node)
                        .is_some_and(|n| n.invalidation.hit_test)
                    {
                        let (bounds, widget) = self
                            .nodes
                            .get(record.node)
                            .map(|n| (n.bounds, n.widget.as_ref()))
                            .unwrap_or((record.bounds, None));

                        let (
                            render_transform_inv,
                            children_render_transform_inv,
                            clips_hit_test,
                            corner_radii,
                        ) = match widget {
                            Some(widget) => (
                                widget.render_transform(bounds).and_then(|t| t.inverse()),
                                widget
                                    .children_render_transform(bounds)
                                    .and_then(|t| t.inverse()),
                                widget.clips_hit_test(bounds),
                                widget.clip_hit_test_corner_radii(bounds),
                            ),
                            None => (None, None, true, None),
                        };

                        let (
                            is_focusable,
                            focus_traversal_children,
                            can_scroll_descendant_into_view,
                        ) = widget
                            .map(|w| {
                                (
                                    w.is_focusable(),
                                    w.focus_traversal_children(),
                                    w.can_scroll_descendant_into_view(),
                                )
                            })
                            .unwrap_or((false, true, false));

                        record.bounds = bounds;
                        record.render_transform_inv = render_transform_inv;
                        record.children_render_transform_inv = children_render_transform_inv;
                        record.clips_hit_test = clips_hit_test;
                        record.clip_hit_test_corner_radii = corner_radii;
                        record.is_focusable = is_focusable;
                        record.focus_traversal_children = focus_traversal_children;
                        record.can_scroll_descendant_into_view = can_scroll_descendant_into_view;
                    }
                    self.interaction_cache.records.push(record);
                    self.apply_interaction_record(&record);
                    self.prepaint_virtual_list_window_from_interaction_record(app, &record);
                }
                let end = self.interaction_cache.records.len();

                if let Some(n) = self.nodes.get_mut(node) {
                    n.interaction_cache = Some(InteractionCacheEntry {
                        generation: self.interaction_cache.target_generation,
                        key,
                        start: start as u32,
                        end: end as u32,
                    });
                }

                self.interaction_cache.hits = self.interaction_cache.hits.saturating_add(1);
                self.interaction_cache.replayed_records = self
                    .interaction_cache
                    .replayed_records
                    .saturating_add((end - start) as u32);
                return;
            }
        }

        if can_reuse {
            self.interaction_cache.misses = self.interaction_cache.misses.saturating_add(1);
        }

        let start = self.interaction_cache.records.len();
        let (render_transform, children_render_transform, clips_hit_test, corner_radii) =
            match self.nodes.get(node).and_then(|n| n.widget.as_ref()) {
                Some(widget) => {
                    let render_transform_inv =
                        widget.render_transform(bounds).and_then(|t| t.inverse());
                    let children_render_transform_inv = widget
                        .children_render_transform(bounds)
                        .and_then(|t| t.inverse());
                    (
                        render_transform_inv,
                        children_render_transform_inv,
                        widget.clips_hit_test(bounds),
                        widget.clip_hit_test_corner_radii(bounds),
                    )
                }
                None => (None, None, true, None),
            };
        let (is_focusable, focus_traversal_children, can_scroll_descendant_into_view) = self
            .nodes
            .get(node)
            .and_then(|n| n.widget.as_ref())
            .map(|w| {
                (
                    w.is_focusable(),
                    w.focus_traversal_children(),
                    w.can_scroll_descendant_into_view(),
                )
            })
            .unwrap_or((false, true, false));

        let record = InteractionRecord {
            node,
            bounds,
            render_transform_inv: render_transform,
            children_render_transform_inv: children_render_transform,
            clips_hit_test,
            clip_hit_test_corner_radii: corner_radii,
            is_focusable,
            focus_traversal_children,
            can_scroll_descendant_into_view,
        };
        self.interaction_cache.records.push(record);
        self.apply_interaction_record(&record);
        self.prepaint_virtual_list_window_from_interaction_record(app, &record);

        let mut children_buf = SmallNodeList::<32>::default();
        if let Some(children) = self.nodes.get(node).map(|n| n.children.as_slice()) {
            children_buf.set(children);
        }
        for &child in children_buf.as_slice() {
            self.prepaint_interaction_node(app, child, scale_factor, theme_revision);
        }

        let end = self.interaction_cache.records.len();
        if is_view_cache_root && let Some(n) = self.nodes.get_mut(node) {
            n.interaction_cache = Some(InteractionCacheEntry {
                generation: self.interaction_cache.target_generation,
                key,
                start: start as u32,
                end: end as u32,
            });
        }
    }
}
