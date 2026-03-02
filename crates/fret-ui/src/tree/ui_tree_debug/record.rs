use super::super::*;

impl<H: UiHost> UiTree<H> {
    pub(crate) fn debug_resolve_layout_solve_root_label(
        &self,
        app: &mut H,
        window: AppWindowId,
        solve_root: NodeId,
    ) -> (
        Option<GlobalElementId>,
        Option<&'static str>,
        Option<String>,
    ) {
        // Layout solve roots may be barrier/layout-only nodes without a corresponding element. For
        // attribution, prefer the nearest ancestor that has an element, so perf bundles can be
        // mapped back to a stable debug path / test_id (when present).
        let mut cur = Some(solve_root);
        while let Some(node) = cur {
            let element = self.nodes.get(node).and_then(|n| n.element);
            if let Some(element) = element {
                let kind = crate::declarative::frame::element_record_for_node(app, window, node)
                    .map(|record| record.instance.kind_name());
                let element_path: Option<String> = {
                    #[cfg(feature = "diagnostics")]
                    {
                        crate::elements::with_window_state(app, window, |st| {
                            st.debug_path_for_element(element)
                        })
                    }
                    #[cfg(not(feature = "diagnostics"))]
                    {
                        let _ = element;
                        None
                    }
                };
                return (Some(element), kind, element_path);
            }

            cur = self.nodes.get(node).and_then(|n| n.parent);
        }

        // Fallback when there is no element in the ancestry (layout-only roots). Prefer the widget
        // type name so perf triage can still point at a meaningful mechanism surface.
        let widget_kind = self
            .nodes
            .get(solve_root)
            .and_then(|n| n.widget.as_ref())
            .map(|w| w.debug_type_name());
        let widget_path = widget_kind.map(|k| format!("widget:{k}"));

        (None, widget_kind, widget_path)
    }

    pub(crate) fn debug_record_hover_edge_pressable(&mut self) {
        if !self.debug_enabled {
            return;
        }
        self.debug_hover_edge_this_frame = true;
        self.debug_stats.hover_pressable_target_changes = self
            .debug_stats
            .hover_pressable_target_changes
            .saturating_add(1);
    }

    pub(crate) fn debug_record_hover_edge_hover_region(&mut self) {
        if !self.debug_enabled {
            return;
        }
        self.debug_hover_edge_this_frame = true;
        self.debug_stats.hover_hover_region_target_changes = self
            .debug_stats
            .hover_hover_region_target_changes
            .saturating_add(1);
    }

    pub(crate) fn hover_edge_changed_this_frame(&self) -> bool {
        self.debug_hover_edge_this_frame
    }

    pub(crate) fn debug_record_hover_declarative_invalidation(
        &mut self,
        node: NodeId,
        hit_test: bool,
        layout: bool,
        paint: bool,
    ) {
        if !self.debug_enabled || !self.debug_hover_edge_this_frame {
            return;
        }

        self.debug_stats.hover_declarative_instance_changes = self
            .debug_stats
            .hover_declarative_instance_changes
            .saturating_add(1);

        self.debug_stats.hover_declarative_hit_test_invalidations = self
            .debug_stats
            .hover_declarative_hit_test_invalidations
            .saturating_add(hit_test as u32);
        self.debug_stats.hover_declarative_layout_invalidations = self
            .debug_stats
            .hover_declarative_layout_invalidations
            .saturating_add(layout as u32);
        self.debug_stats.hover_declarative_paint_invalidations = self
            .debug_stats
            .hover_declarative_paint_invalidations
            .saturating_add(paint as u32);

        let entry = self
            .debug_hover_declarative_invalidations
            .entry(node)
            .or_default();
        entry.hit_test = entry.hit_test.saturating_add(hit_test as u32);
        entry.layout = entry.layout.saturating_add(layout as u32);
        entry.paint = entry.paint.saturating_add(paint as u32);
    }

    pub(crate) fn debug_record_measure_child(
        &mut self,
        parent: NodeId,
        child: NodeId,
        elapsed: Duration,
    ) {
        if !self.debug_enabled {
            return;
        }
        let entry = self
            .debug_measure_children
            .entry(parent)
            .or_default()
            .entry(child)
            .or_default();
        entry.total_time += elapsed;
        entry.calls = entry.calls.saturating_add(1);
    }

    pub(crate) fn debug_record_text_constraints_measured(
        &mut self,
        node: NodeId,
        constraints: TextConstraints,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            if self.debug_enabled {
                self.debug_text_constraints_measured
                    .insert(node, constraints);
            }
        }
        #[cfg(not(feature = "diagnostics"))]
        {
            let _ = (node, constraints);
        }
    }

    pub(crate) fn debug_record_text_constraints_prepared(
        &mut self,
        node: NodeId,
        constraints: TextConstraints,
    ) {
        #[cfg(feature = "diagnostics")]
        {
            if self.debug_enabled {
                self.debug_text_constraints_prepared
                    .insert(node, constraints);
            }
        }
        #[cfg(not(feature = "diagnostics"))]
        {
            let _ = (node, constraints);
        }
    }

    pub(in crate::tree) fn debug_take_top_measure_children(
        &mut self,
        parent: NodeId,
        max: usize,
    ) -> Vec<(NodeId, DebugMeasureChildRecord)> {
        let Some(children) = self.debug_measure_children.remove(&parent) else {
            return Vec::new();
        };
        let mut items: Vec<(NodeId, DebugMeasureChildRecord)> = children.into_iter().collect();
        items.sort_by_key(|(_, r)| std::cmp::Reverse(r.total_time));
        items.truncate(max);
        items
    }

    pub(crate) fn debug_record_view_cache_root(
        &mut self,
        root: NodeId,
        reused: bool,
        contained_layout: bool,
        reuse_reason: UiDebugCacheRootReuseReason,
    ) {
        if !self.debug_enabled {
            return;
        }
        self.debug_stats.view_cache_roots_total =
            self.debug_stats.view_cache_roots_total.saturating_add(1);
        if reused {
            self.debug_stats.view_cache_roots_reused =
                self.debug_stats.view_cache_roots_reused.saturating_add(1);
        }
        match reuse_reason {
            UiDebugCacheRootReuseReason::FirstMount => {
                self.debug_stats.view_cache_roots_first_mount = self
                    .debug_stats
                    .view_cache_roots_first_mount
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::NodeRecreated => {
                self.debug_stats.view_cache_roots_node_recreated = self
                    .debug_stats
                    .view_cache_roots_node_recreated
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::CacheKeyMismatch => {
                self.debug_stats.view_cache_roots_cache_key_mismatch = self
                    .debug_stats
                    .view_cache_roots_cache_key_mismatch
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::ViewCacheDisabled
            | UiDebugCacheRootReuseReason::InspectionActive => {}
            UiDebugCacheRootReuseReason::NotMarkedReuseRoot => {
                self.debug_stats.view_cache_roots_not_marked_reuse_root = self
                    .debug_stats
                    .view_cache_roots_not_marked_reuse_root
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::NeedsRerender => {
                self.debug_stats.view_cache_roots_needs_rerender = self
                    .debug_stats
                    .view_cache_roots_needs_rerender
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::LayoutInvalidated => {
                self.debug_stats.view_cache_roots_layout_invalidated = self
                    .debug_stats
                    .view_cache_roots_layout_invalidated
                    .saturating_add(1);
            }
            UiDebugCacheRootReuseReason::ManualCacheRoot => {
                self.debug_stats.view_cache_roots_manual =
                    self.debug_stats.view_cache_roots_manual.saturating_add(1);
            }
            UiDebugCacheRootReuseReason::MarkedReuseRoot => {}
        }
        self.debug_view_cache_roots.push(DebugViewCacheRootRecord {
            root,
            reused,
            contained_layout,
            reuse_reason,
        });
    }

    pub(crate) fn debug_record_virtual_list_window(&mut self, record: UiDebugVirtualListWindow) {
        if !self.debug_enabled {
            return;
        }
        if record.window_shift_kind != UiDebugVirtualListWindowShiftKind::None {
            self.debug_stats.virtual_list_window_shifts_total = self
                .debug_stats
                .virtual_list_window_shifts_total
                .saturating_add(1);
            if record.window_shift_apply_mode
                == Some(UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender)
            {
                self.debug_stats.virtual_list_window_shifts_non_retained = self
                    .debug_stats
                    .virtual_list_window_shifts_non_retained
                    .saturating_add(1);
            }
        }

        if record.window_shift_apply_mode
            == Some(UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender)
            && record.window_shift_kind != UiDebugVirtualListWindowShiftKind::None
            && let Some(reason) = record.window_shift_reason
        {
            // Keep bundles bounded: window shifts can occur frequently during scroll.
            const MAX_SAMPLES: usize = 64;
            if self.debug_virtual_list_window_shift_samples.len() < MAX_SAMPLES {
                self.debug_virtual_list_window_shift_samples.push(
                    UiDebugVirtualListWindowShiftSample {
                        frame_id: self.debug_stats.frame_id,
                        source: record.source,
                        node: record.node,
                        element: record.element,
                        window_shift_kind: record.window_shift_kind,
                        window_shift_reason: reason,
                        window_shift_apply_mode:
                            UiDebugVirtualListWindowShiftApplyMode::NonRetainedRerender,
                        window_shift_invalidation_detail: record.window_shift_invalidation_detail,
                        prev_window_range: record.prev_window_range,
                        window_range: record.window_range,
                        render_window_range: record.render_window_range,
                    },
                );
            }
        }
        // Keep bundles bounded: real apps can have many virtual surfaces.
        const MAX_RECORDS: usize = 256;
        if self.debug_virtual_list_windows.len() >= MAX_RECORDS {
            return;
        }
        self.debug_virtual_list_windows.push(record);
    }

    pub(crate) fn debug_record_scroll_node_telemetry(
        &mut self,
        record: UiDebugScrollNodeTelemetry,
    ) {
        if !self.debug_enabled {
            return;
        }
        // Keep bundles bounded: real apps can have many scroll containers.
        const MAX_RECORDS: usize = 256;
        if self.debug_scroll_nodes.len() >= MAX_RECORDS {
            return;
        }
        self.debug_scroll_nodes.push(record);
    }

    pub(crate) fn debug_record_scrollbar_telemetry(&mut self, record: UiDebugScrollbarTelemetry) {
        if !self.debug_enabled {
            return;
        }
        // Keep bundles bounded: real apps can have many scrollbars.
        const MAX_RECORDS: usize = 256;
        if self.debug_scrollbars.len() >= MAX_RECORDS {
            return;
        }
        self.debug_scrollbars.push(record);
    }

    pub(crate) fn debug_record_retained_virtual_list_reconcile(
        &mut self,
        record: UiDebugRetainedVirtualListReconcile,
    ) {
        if !self.debug_enabled {
            return;
        }
        const MAX_RECORDS: usize = 128;
        if self.debug_retained_virtual_list_reconciles.len() >= MAX_RECORDS {
            return;
        }
        self.debug_stats.retained_virtual_list_reconciles = self
            .debug_stats
            .retained_virtual_list_reconciles
            .saturating_add(1);
        self.debug_stats.retained_virtual_list_attached_items = self
            .debug_stats
            .retained_virtual_list_attached_items
            .saturating_add(record.attached_items);
        self.debug_stats.retained_virtual_list_detached_items = self
            .debug_stats
            .retained_virtual_list_detached_items
            .saturating_add(record.detached_items);
        self.debug_retained_virtual_list_reconciles.push(record);
    }

    pub(crate) fn debug_record_prepaint_action(&mut self, action: UiDebugPrepaintAction) {
        if !self.debug_enabled {
            return;
        }
        // Keep bundles bounded: real apps can have many prepaint actions.
        const MAX_RECORDS: usize = 512;
        if self.debug_prepaint_actions.len() >= MAX_RECORDS {
            return;
        }
        self.debug_prepaint_actions.push(action);
    }

    pub(crate) fn debug_record_paint_cache_replay(&mut self, node: NodeId, replayed_ops: u32) {
        if !self.debug_enabled {
            return;
        }
        *self.debug_paint_cache_replays.entry(node).or_default() += replayed_ops;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn debug_record_layout_engine_solve(
        &mut self,
        root: NodeId,
        root_element: Option<GlobalElementId>,
        root_element_kind: Option<&'static str>,
        root_element_path: Option<String>,
        solve_time: Duration,
        measure_calls: u64,
        measure_cache_hits: u64,
        measure_time: Duration,
        top_measures: Vec<UiDebugLayoutEngineMeasureHotspot>,
    ) {
        if !self.debug_enabled {
            return;
        }
        // Keep bundles bounded: barrier layouts may solve many child roots in a single frame.
        const MAX_LAYOUT_ENGINE_SOLVES: usize = 16;
        let record = UiDebugLayoutEngineSolve {
            root,
            root_element,
            root_element_kind,
            root_element_path,
            solve_time,
            measure_calls,
            measure_cache_hits,
            measure_time,
            top_measures,
        };

        let idx = self
            .debug_layout_engine_solves
            .iter()
            .position(|h| h.solve_time < record.solve_time)
            .unwrap_or(self.debug_layout_engine_solves.len());
        self.debug_layout_engine_solves.insert(idx, record);
        if self.debug_layout_engine_solves.len() > MAX_LAYOUT_ENGINE_SOLVES {
            self.debug_layout_engine_solves
                .truncate(MAX_LAYOUT_ENGINE_SOLVES);
        }
    }
}
