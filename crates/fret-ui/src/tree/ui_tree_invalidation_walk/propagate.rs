use super::super::*;

impl<H: UiHost> UiTree<H> {
    /// Convenience helper for single-window/single-tree setups.
    ///
    /// This drains pending model changes from the host and immediately propagates them into the
    /// tree. Multi-window runtimes should drain `take_changed_models()` once per frame and fan the
    /// resulting list out to each window's [`UiTree`] instead.
    pub fn propagate_pending_model_changes(&mut self, app: &mut H) -> bool {
        let changed = app.take_changed_models();
        self.propagate_model_changes(app, &changed)
    }

    fn propagate_observation_masks(
        &mut self,
        app: &mut H,
        masks: impl IntoIterator<Item = (NodeId, ObservationMask)>,
        source: UiDebugInvalidationSource,
    ) -> bool {
        self.propagation_depth_generation = self.propagation_depth_generation.wrapping_add(1);
        if self.propagation_depth_generation == 0 {
            self.propagation_depth_generation = 1;
            self.propagation_depth_cache.clear();
        }
        self.propagation_chain.clear();
        self.propagation_entries.clear();

        for (node, mask) in masks {
            if mask.is_empty() || !self.nodes.contains_key(node) {
                continue;
            }

            let (strength, inv) = if mask.hit_test {
                (3, Invalidation::HitTest)
            } else if mask.layout {
                (2, Invalidation::Layout)
            } else if mask.paint {
                (1, Invalidation::Paint)
            } else {
                continue;
            };

            let depth = propagation_depth::propagation_depth_for(self, node);
            let key = node.data().as_ffi();
            self.propagation_entries
                .push((strength, depth, key, node, inv));
        }

        if self.propagation_entries.is_empty() {
            return false;
        }

        self.propagation_entries.sort_by(|a, b| {
            // Higher-strength invalidations first to maximize reuse via `visited`.
            b.0.cmp(&a.0)
                // Within the same strength, prefer ancestors first to reduce redundant walks.
                .then(a.1.cmp(&b.1))
                // Stabilize order for determinism in stats/perf.
                .then(a.2.cmp(&b.2))
        });

        let mut did_invalidate = false;
        let mut visited = std::mem::take(&mut self.invalidation_dedup);
        visited.begin();
        let mut entries = std::mem::take(&mut self.propagation_entries);
        for (_, _, _, node, inv) in entries.drain(..) {
            self.mark_invalidation_dedup_with_source(node, inv, &mut visited, source);
            did_invalidate = true;
        }
        self.invalidation_dedup = visited;
        self.propagation_entries = entries;

        if did_invalidate {
            self.request_redraw_coalesced(app);
        }

        did_invalidate
    }

    fn propagate_model_changes_from_elements(&mut self, app: &mut H, changed: &[ModelId]) -> bool {
        let Some(window) = self.window else {
            return false;
        };
        if changed.is_empty() {
            return false;
        }

        let changed: std::collections::HashSet<ModelId> = changed.iter().copied().collect();
        let frame_id = app.frame_id();
        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, _app| {
            let Some(window_state) = runtime.for_window(window) else {
                return;
            };
            window_state.for_each_observed_model_for_invalidation(
                frame_id,
                |element, observations| {
                    let mut mask = ObservationMask::default();
                    for (model, inv) in observations {
                        if changed.contains(model) {
                            mask.add(*inv);
                        }
                    }
                    if mask.is_empty() {
                        return;
                    }
                    let Some(node) = window_state.node_entry(element).map(|e| e.node) else {
                        return;
                    };
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                },
            );
        });

        if combined.is_empty() {
            return false;
        }
        self.propagate_observation_masks(app, combined, UiDebugInvalidationSource::ModelChange)
    }

    fn propagate_global_changes_from_elements(&mut self, app: &mut H, changed: &[TypeId]) -> bool {
        let Some(window) = self.window else {
            return false;
        };
        if changed.is_empty() {
            return false;
        }

        let changed: std::collections::HashSet<TypeId> = changed.iter().copied().collect();
        let frame_id = app.frame_id();
        let mut combined: HashMap<NodeId, ObservationMask> = HashMap::new();

        app.with_global_mut_untracked(crate::elements::ElementRuntime::new, |runtime, _app| {
            let Some(window_state) = runtime.for_window(window) else {
                return;
            };
            window_state.for_each_observed_global_for_invalidation(
                frame_id,
                |element, observations| {
                    let mut mask = ObservationMask::default();
                    for (global, inv) in observations {
                        if changed.contains(global) {
                            mask.add(*inv);
                        }
                    }
                    if mask.is_empty() {
                        return;
                    }
                    let Some(node) = window_state.node_entry(element).map(|e| e.node) else {
                        return;
                    };
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                },
            );
        });

        if combined.is_empty() {
            return false;
        }
        self.propagate_observation_masks(app, combined, UiDebugInvalidationSource::GlobalChange)
    }

    pub fn propagate_model_changes(&mut self, app: &mut H, changed: &[ModelId]) -> bool {
        if changed.is_empty() {
            return false;
        }
        self.begin_debug_frame_if_needed(app.frame_id());
        if self.debug_enabled {
            self.debug_model_change_hotspots.clear();
            self.debug_model_change_unobserved.clear();
        }

        let mut did_invalidate = false;

        if changed.len() == 1 {
            let model = changed[0];
            let layout_nodes = self.observed_in_layout.by_model.get(&model);
            let paint_nodes = self.observed_in_paint.by_model.get(&model);
            if let (Some(nodes), None) | (None, Some(nodes)) = (layout_nodes, paint_nodes) {
                // Copy out the observations so we don't hold a borrow across the invalidation walk.
                let masks: Vec<(NodeId, ObservationMask)> =
                    nodes.iter().map(|(&n, &m)| (n, m)).collect();
                if self.debug_enabled {
                    self.debug_stats.model_change_invalidation_roots =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.model_change_models = 1;
                    self.debug_stats.model_change_observation_edges =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.model_change_unobserved_models = 0;
                    self.debug_model_change_hotspots = vec![UiDebugModelChangeHotspot {
                        model,
                        observation_edges: masks.len().min(u32::MAX as usize) as u32,
                        changed: app.models().debug_last_changed_info_for_id(model),
                    }];
                }
                did_invalidate |= self.propagate_observation_masks(
                    app,
                    masks,
                    UiDebugInvalidationSource::ModelChange,
                );
                did_invalidate |= self.propagate_model_changes_from_elements(app, changed);
                return did_invalidate;
            }
        }

        // Avoid rehash spikes: `changed` is usually small while each changed model/global can have
        // thousands of observation edges.
        let mut combined_capacity = 0usize;
        for &model in changed {
            if let Some(nodes) = self.observed_in_layout.by_model.get(&model) {
                combined_capacity = combined_capacity.saturating_add(nodes.len());
            }
            if let Some(nodes) = self.observed_in_paint.by_model.get(&model) {
                combined_capacity = combined_capacity.saturating_add(nodes.len());
            }
        }
        combined_capacity = combined_capacity.min(self.nodes.len());

        let mut combined: HashMap<NodeId, ObservationMask> =
            HashMap::with_capacity(combined_capacity.max(changed.len().saturating_mul(8)));
        let mut observation_edges_scanned = 0usize;
        let mut unobserved_models = 0usize;
        for &model in changed {
            let mut edges = 0usize;
            if let Some(nodes) = self.observed_in_layout.by_model.get(&model) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_in_paint.by_model.get(&model) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if self.debug_enabled && edges > 0 {
                self.debug_model_change_hotspots
                    .push(UiDebugModelChangeHotspot {
                        model,
                        observation_edges: edges.min(u32::MAX as usize) as u32,
                        changed: app.models().debug_last_changed_info_for_id(model),
                    });
            }
            if edges == 0 {
                unobserved_models = unobserved_models.saturating_add(1);
                if self.debug_enabled {
                    self.debug_model_change_unobserved
                        .push(UiDebugModelChangeUnobserved {
                            model,
                            created: app.models().debug_created_info_for_id(model),
                            changed: app.models().debug_last_changed_info_for_id(model),
                        });
                }
            }
        }

        if self.debug_enabled {
            self.debug_stats.model_change_invalidation_roots =
                combined.len().min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_models = changed.len().min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_observation_edges =
                observation_edges_scanned.min(u32::MAX as usize) as u32;
            self.debug_stats.model_change_unobserved_models =
                unobserved_models.min(u32::MAX as usize) as u32;

            self.debug_model_change_hotspots
                .sort_by(|a, b| b.observation_edges.cmp(&a.observation_edges));
            self.debug_model_change_hotspots.truncate(5);

            self.debug_model_change_unobserved
                .sort_by(|a, b| a.model.data().as_ffi().cmp(&b.model.data().as_ffi()));
            self.debug_model_change_unobserved.truncate(5);
        }
        did_invalidate |=
            self.propagate_observation_masks(app, combined, UiDebugInvalidationSource::ModelChange);
        did_invalidate |= self.propagate_model_changes_from_elements(app, changed);
        did_invalidate
    }

    pub fn propagate_global_changes(&mut self, app: &mut H, changed: &[TypeId]) -> bool {
        if changed.is_empty() {
            return false;
        }
        self.begin_debug_frame_if_needed(app.frame_id());
        if self.debug_enabled {
            self.debug_global_change_hotspots.clear();
            self.debug_global_change_unobserved.clear();
        }

        let mut did_invalidate = false;

        if changed.len() == 1 {
            let global = changed[0];
            let layout_nodes = self.observed_globals_in_layout.by_global.get(&global);
            let paint_nodes = self.observed_globals_in_paint.by_global.get(&global);
            if let (Some(nodes), None) | (None, Some(nodes)) = (layout_nodes, paint_nodes) {
                // Copy out the observations so we don't hold a borrow across the invalidation walk.
                let masks: Vec<(NodeId, ObservationMask)> =
                    nodes.iter().map(|(&n, &m)| (n, m)).collect();
                if self.debug_enabled {
                    self.debug_stats.global_change_invalidation_roots =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.global_change_globals = 1;
                    self.debug_stats.global_change_observation_edges =
                        masks.len().min(u32::MAX as usize) as u32;
                    self.debug_stats.global_change_unobserved_globals = 0;
                }
                did_invalidate |= self.propagate_observation_masks(
                    app,
                    masks,
                    UiDebugInvalidationSource::GlobalChange,
                );
                did_invalidate |= self.propagate_global_changes_from_elements(app, changed);
                return did_invalidate;
            }
        }

        // Avoid rehash spikes: `changed` is usually small while each changed global can have
        // thousands of observation edges.
        let mut combined_capacity = 0usize;
        for &global in changed {
            if let Some(nodes) = self.observed_globals_in_layout.by_global.get(&global) {
                combined_capacity = combined_capacity.saturating_add(nodes.len());
            }
            if let Some(nodes) = self.observed_globals_in_paint.by_global.get(&global) {
                combined_capacity = combined_capacity.saturating_add(nodes.len());
            }
        }
        combined_capacity = combined_capacity.min(self.nodes.len());

        let mut combined: HashMap<NodeId, ObservationMask> =
            HashMap::with_capacity(combined_capacity.max(changed.len().saturating_mul(8)));
        let mut observation_edges_scanned = 0usize;
        let mut unobserved_globals = 0usize;
        for &global in changed {
            let mut edges = 0usize;
            if let Some(nodes) = self.observed_globals_in_layout.by_global.get(&global) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if let Some(nodes) = self.observed_globals_in_paint.by_global.get(&global) {
                observation_edges_scanned = observation_edges_scanned.saturating_add(nodes.len());
                edges = edges.saturating_add(nodes.len());
                for (&node, &mask) in nodes {
                    combined
                        .entry(node)
                        .and_modify(|m| *m = m.union(mask))
                        .or_insert(mask);
                }
            }
            if self.debug_enabled && edges > 0 {
                self.debug_global_change_hotspots
                    .push(UiDebugGlobalChangeHotspot {
                        global,
                        observation_edges: edges.min(u32::MAX as usize) as u32,
                    });
            }
            if edges == 0 {
                unobserved_globals = unobserved_globals.saturating_add(1);
                if self.debug_enabled {
                    self.debug_global_change_unobserved
                        .push(UiDebugGlobalChangeUnobserved { global });
                }
            }
        }

        if self.debug_enabled {
            self.debug_stats.global_change_invalidation_roots =
                combined.len().min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_globals = changed.len().min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_observation_edges =
                observation_edges_scanned.min(u32::MAX as usize) as u32;
            self.debug_stats.global_change_unobserved_globals =
                unobserved_globals.min(u32::MAX as usize) as u32;

            self.debug_global_change_hotspots
                .sort_by(|a, b| b.observation_edges.cmp(&a.observation_edges));
            self.debug_global_change_hotspots.truncate(5);

            self.debug_global_change_unobserved
                .sort_by_key(|u| type_id_sort_key(u.global));
            self.debug_global_change_unobserved.truncate(5);
        }
        did_invalidate |= self.propagate_observation_masks(
            app,
            combined,
            UiDebugInvalidationSource::GlobalChange,
        );
        did_invalidate |= self.propagate_global_changes_from_elements(app, changed);
        did_invalidate
    }
}
