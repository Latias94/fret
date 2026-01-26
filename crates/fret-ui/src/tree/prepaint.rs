use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct InteractionCacheEntry {
    pub(super) generation: u64,
    pub(super) key: PaintCacheKey,
    pub(super) start: u32,
    pub(super) end: u32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub(super) struct InteractionRecord {
    pub(super) node: NodeId,
    pub(super) bounds: Rect,
    pub(super) render_transform_inv: Option<Transform2D>,
    pub(super) children_render_transform_inv: Option<Transform2D>,
    pub(super) clips_hit_test: bool,
    pub(super) clip_hit_test_corner_radii: Option<Corners>,
    pub(super) is_focusable: bool,
    pub(super) focus_traversal_children: bool,
    pub(super) can_scroll_descendant_into_view: bool,
}

#[derive(Debug, Default)]
pub(super) struct InteractionCacheState {
    generation: u64,
    pub(super) prev_records: Vec<InteractionRecord>,
    pub(super) records: Vec<InteractionRecord>,
    pub(super) source_generation: u64,
    pub(super) target_generation: u64,
    pub(super) hits: u32,
    pub(super) misses: u32,
    pub(super) replayed_records: u32,
}

impl InteractionCacheState {
    pub(super) fn begin_frame(&mut self) {
        self.source_generation = self.generation;
        self.target_generation = self.generation.saturating_add(1);
        self.hits = 0;
        self.misses = 0;
        self.replayed_records = 0;

        std::mem::swap(&mut self.prev_records, &mut self.records);
        self.records.clear();
    }

    pub(super) fn finish_frame(&mut self) {
        self.generation = self.target_generation;
    }

    pub(super) fn invalidate_recording(&mut self) {
        self.prev_records.clear();
        self.records.clear();
        self.generation = self.generation.saturating_add(1);
    }
}

impl<H: UiHost> UiTree<H> {
    fn apply_interaction_record(&mut self, record: &InteractionRecord) {
        let Some(n) = self.nodes.get_mut(record.node) else {
            return;
        };
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
    }

    pub(super) fn prepaint_after_layout(&mut self, app: &mut H, scale_factor: f32) {
        if self.inspection_active {
            self.interaction_cache.invalidate_recording();
            return;
        }

        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.prepaint_time = Duration::default();
            self.debug_stats.prepaint_nodes_visited = 0;
            self.debug_stats.interaction_cache_hits = 0;
            self.debug_stats.interaction_cache_misses = 0;
            self.debug_stats.interaction_cache_replayed_records = 0;
            self.debug_stats.interaction_records = 0;
        }

        self.interaction_cache.begin_frame();

        let theme_revision = Theme::global(&*app).revision();
        let roots: Vec<NodeId> = self
            .visible_layers_in_paint_order()
            .map(|layer| self.layers[layer].root)
            .collect();
        for root in roots {
            self.prepaint_interaction_node(app, root, scale_factor, theme_revision);
        }

        self.interaction_cache.finish_frame();
        if self.debug_enabled {
            self.debug_stats.interaction_cache_hits = self.interaction_cache.hits;
            self.debug_stats.interaction_cache_misses = self.interaction_cache.misses;
            self.debug_stats.interaction_cache_replayed_records =
                self.interaction_cache.replayed_records;
            self.debug_stats.interaction_records = self.interaction_cache.records.len() as u32;
        }
        if let Some(started) = started {
            self.debug_stats.prepaint_time = started.elapsed();
        }
    }

    fn prepaint_interaction_node(
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

        let (bounds, invalidation, is_view_cache_root, prev_cache, is_manual_cache_root) =
            match self.nodes.get(node) {
                Some(n) => (
                    n.bounds,
                    n.invalidation,
                    self.view_cache_active() && n.view_cache.enabled,
                    n.interaction_cache,
                    n.view_cache.enabled && n.element.is_none(),
                ),
                None => return,
            };

        let child_transform = self
            .node_children_render_transform(node)
            .unwrap_or(Transform2D::IDENTITY);
        let key = PaintCacheKey::new(bounds, scale_factor, theme_revision, child_transform);

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

        if is_view_cache_root {
            let window = self.window;
            let sf = scale_factor;
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
                let replay: Vec<InteractionRecord> =
                    self.interaction_cache.prev_records[range].to_vec();
                for record in &replay {
                    self.interaction_cache.records.push(*record);
                    self.apply_interaction_record(record);
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

        let mut children_buf = SmallNodeList::<32>::default();
        if let Some(children) = self.nodes.get(node).map(|n| n.children.as_slice()) {
            children_buf.set(children);
        }
        for &child in children_buf.as_slice() {
            self.prepaint_interaction_node(app, child, scale_factor, theme_revision);
        }

        let end = self.interaction_cache.records.len();
        if is_view_cache_root {
            if let Some(n) = self.nodes.get_mut(node) {
                n.interaction_cache = Some(InteractionCacheEntry {
                    generation: self.interaction_cache.target_generation,
                    key,
                    start: start as u32,
                    end: end as u32,
                });
            }
        }
    }
}
