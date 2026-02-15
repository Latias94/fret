use super::*;

impl<H: UiHost> UiTree<H> {
    pub(in crate::tree) fn prepaint_after_layout(&mut self, app: &mut H, scale_factor: f32) {
        if self.inspection_active {
            self.interaction_cache.invalidate_recording();
            self.hit_test_bounds_trees.clear();
            return;
        }

        let started = self.debug_enabled.then(Instant::now);
        if self.debug_enabled {
            self.begin_debug_frame_if_needed(app.frame_id());
            self.debug_stats.prepaint_time = Duration::default();
            self.debug_stats.prepaint_nodes_visited = 0;
            // Stable-frame prepaint reuses the previously recorded interaction stream without
            // rebuilding it. Surface reuse explicitly so tests and perf tooling can still observe
            // that hit-test relevant metadata remains cached.
            self.debug_stats.interaction_cache_hits = self.interaction_cache.records.len() as u32;
            self.debug_stats.interaction_cache_misses = 0;
            self.debug_stats.interaction_cache_replayed_records = 0;
            self.debug_stats.interaction_records = self.interaction_cache.records.len() as u32;
        }

        self.interaction_cache.begin_frame();
        self.hit_test_bounds_trees.begin_frame(app.frame_id());

        let theme_revision = Theme::global(&*app).revision();
        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers {
            let root = self.layers[layer_id].root;
            let hit_testable = self.layers[layer_id].hit_testable;

            let start = self.interaction_cache.records.len();
            self.prepaint_interaction_node(app, root, scale_factor, theme_revision);
            let end = self.interaction_cache.records.len();

            if hit_testable {
                let records = &self.interaction_cache.records[start..end];
                let nodes = &self.nodes;
                self.hit_test_bounds_trees
                    .rebuild_for_layer_from_records(root, records, nodes);
            }
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

    pub(in crate::tree) fn prepaint_after_layout_stable_frame(&mut self, app: &mut H) {
        if self.inspection_active {
            self.interaction_cache.invalidate_recording();
            self.hit_test_bounds_trees.clear();
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

        self.hit_test_bounds_trees.begin_frame(app.frame_id());

        let layers: Vec<UiLayerId> = self.visible_layers_in_paint_order().collect();
        for layer_id in layers {
            let root = self.layers[layer_id].root;
            let hit_testable = self.layers[layer_id].hit_testable;
            if hit_testable {
                self.hit_test_bounds_trees.reuse_for_layer(root);
            }
        }

        if let Some(started) = started {
            self.debug_stats.prepaint_time = started.elapsed();
        }
    }
}
