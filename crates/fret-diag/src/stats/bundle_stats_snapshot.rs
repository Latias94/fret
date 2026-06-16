use super::{
    BundleStatsCleanGeometrySolveSkipRejection, BundleStatsCommandAvailabilityHotspot,
    BundleStatsGlobalChangeHotspot, BundleStatsGlobalChangeUnobserved,
    BundleStatsLayoutDirtyDescendant, BundleStatsLayoutEngineMeasureChildHotspot,
    BundleStatsLayoutEngineMeasureHotspot, BundleStatsLayoutEngineSolve,
    BundleStatsLayoutEngineSolveProfile, BundleStatsLayoutHotspot,
    BundleStatsLayoutRequestBuildRoot, BundleStatsModelChangeHotspot,
    BundleStatsModelChangeUnobserved, BundleStatsPaintTextPrepareHotspot,
    BundleStatsPaintWidgetHotspot, BundleStatsScrollLayoutKindProfile,
    BundleStatsScrollLayoutPhaseProfile, BundleStatsScrollLayoutProfile,
    BundleStatsWidgetMeasureHotspot,
};

pub(super) fn snapshot_paint_widget_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintWidgetHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_widget_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsPaintWidgetHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintWidgetHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            paint_time_us: h.get("paint_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_scene_ops_delta: h
                .get("inclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            exclusive_scene_ops_delta: h
                .get("exclusive_scene_ops_delta")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_layout_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsLayoutHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsLayoutHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            layout_time_us: h
                .get("layout_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_layout_request_build_roots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutRequestBuildRoot> {
    let roots = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_request_build_roots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if roots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsLayoutRequestBuildRoot> = roots
        .iter()
        .take(max.max(1))
        .map(|r| BundleStatsLayoutRequestBuildRoot {
            root_node: r.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
            root_kind: r
                .get("root_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            root_element: r.get("root_element").and_then(|v| v.as_u64()),
            root_element_kind: r
                .get("root_element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            root_element_path: r
                .get("root_element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            elapsed_us: r.get("elapsed_us").and_then(|v| v.as_u64()).unwrap_or(0),
            mode: r
                .get("mode")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            had_layout_engine_node: r
                .get("had_layout_engine_node")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            layout_invalidated: r
                .get("layout_invalidated")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            subtree_layout_dirty: r
                .get("subtree_layout_dirty")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            subtree_layout_dirty_count: r
                .get("subtree_layout_dirty_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            descendant_layout_dirty_count: r
                .get("descendant_layout_dirty_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            needs_layout: r
                .get("needs_layout")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_translation_only: r
                .get("is_translation_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            nodes_marked_seen: r
                .get("nodes_marked_seen")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            dirty_descendants: r
                .get("dirty_descendants")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .take(4)
                        .map(|d| BundleStatsLayoutDirtyDescendant {
                            node: d.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                            element: d.get("element").and_then(|v| v.as_u64()),
                            element_kind: d
                                .get("element_kind")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            element_path: d
                                .get("element_path")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            subtree_layout_dirty_count: d
                                .get("subtree_layout_dirty_count")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0)
                                .min(u32::MAX as u64)
                                as u32,
                            source_root_node: d.get("source_root_node").and_then(|v| v.as_u64()),
                            source: d
                                .get("source")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            detail: d
                                .get("detail")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            role: None,
                            test_id: None,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            root_role: None,
            root_test_id: None,
        })
        .collect();

    out.sort_by(|a, b| b.elapsed_us.cmp(&a.elapsed_us));
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.root_node);
        item.root_role = role;
        item.root_test_id = test_id;
        for dirty in &mut item.dirty_descendants {
            let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(dirty.node);
            dirty.role = role;
            dirty.test_id = test_id;
        }
    }

    out
}

pub(super) fn snapshot_scroll_layout_profiles(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsScrollLayoutProfile> {
    let scroll_nodes = snapshot
        .get("debug")
        .and_then(|v| v.get("scroll_nodes"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if scroll_nodes.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let parse_kind_profiles = |p: &serde_json::Map<String, serde_json::Value>,
                               key: &str|
     -> Vec<BundleStatsScrollLayoutKindProfile> {
        p.get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_object())
                    .map(|item| BundleStatsScrollLayoutKindProfile {
                        kind: item
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        nodes: item
                            .get("nodes")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0)
                            .min(u32::MAX as u64) as u32,
                        self_us: item.get("self_us").and_then(|v| v.as_u64()).unwrap_or(0),
                        total_us: item.get("total_us").and_then(|v| v.as_u64()).unwrap_or(0),
                        max_self_us: item
                            .get("max_self_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        max_total_us: item
                            .get("max_total_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let parse_phase_profiles = |p: &serde_json::Map<String, serde_json::Value>,
                                key: &str|
     -> Vec<BundleStatsScrollLayoutPhaseProfile> {
        p.get(key)
            .and_then(|v| v.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_object())
                    .map(|item| BundleStatsScrollLayoutPhaseProfile {
                        phase: item
                            .get("phase")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        us: item.get("us").and_then(|v| v.as_u64()).unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let mut out: Vec<BundleStatsScrollLayoutProfile> = scroll_nodes
        .iter()
        .filter_map(|n| {
            let p = n.get("layout_profile").and_then(|v| v.as_object())?;
            Some(BundleStatsScrollLayoutProfile {
                node: n.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                element: n.get("element").and_then(|v| v.as_u64()),
                test_id: n
                    .get("test_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                axis: n
                    .get("axis")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                pass: p
                    .get("pass")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                probe_unbounded: p
                    .get("probe_unbounded")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                children: p
                    .get("children")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                available_w: p
                    .get("available_w")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                available_h: p
                    .get("available_h")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                desired_w: p
                    .get("desired_w")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                desired_h: p
                    .get("desired_h")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                content_w: p
                    .get("content_w")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                content_h: p
                    .get("content_h")
                    .and_then(|v| v.as_f64())
                    .map(|v| v as f32),
                post_layout_extents_mode: p
                    .get("post_layout_extents_mode")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                interactive_resize: p
                    .get("interactive_resize")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                direct_children_layout_invalidated: p
                    .get("direct_children_layout_invalidated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                descendant_subtree_layout_dirty: p
                    .get("descendant_subtree_layout_dirty")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                force_barrier_child_root_relayout: p
                    .get("force_barrier_child_root_relayout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                phase_profiles: parse_phase_profiles(p, "phase_profiles"),
                measure_children_us: p
                    .get("measure_children_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                solve_barrier_us: p
                    .get("solve_barrier_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_children_us: p
                    .get("layout_children_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_children_first_pass_us: p
                    .get("layout_children_first_pass_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_child_first_pass_roots: p
                    .get("layout_child_first_pass_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_first_pass_layout_invalidated_roots: p
                    .get("layout_child_first_pass_layout_invalidated_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_subtree_dirty_roots: p
                    .get("layout_child_first_pass_subtree_dirty_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_clean_roots: p
                    .get("layout_child_first_pass_clean_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_performed_roots: p
                    .get("layout_child_first_pass_performed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_skipped_roots: p
                    .get("layout_child_first_pass_skipped_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_bounds_changed_roots: p
                    .get("layout_child_first_pass_bounds_changed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_bounds_size_changed_roots: p
                    .get("layout_child_first_pass_bounds_size_changed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_input_mismatch_roots: p
                    .get("layout_child_first_pass_input_mismatch_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_input_size_mismatch_roots: p
                    .get("layout_child_first_pass_input_size_mismatch_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_nodes_visited: p
                    .get("layout_child_first_pass_nodes_visited")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_nodes_performed: p
                    .get("layout_child_first_pass_nodes_performed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_first_pass_max_us: p
                    .get("layout_child_first_pass_max_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_child_first_pass_kind_profiles: parse_kind_profiles(
                    p,
                    "layout_child_first_pass_kind_profiles",
                ),
                corrected_content_relayout: p
                    .get("corrected_content_relayout")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                layout_children_corrected_content_us: p
                    .get("layout_children_corrected_content_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_child_corrected_content_nodes_visited: p
                    .get("layout_child_corrected_content_nodes_visited")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_corrected_content_nodes_performed: p
                    .get("layout_child_corrected_content_nodes_performed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_corrected_content_max_us: p
                    .get("layout_child_corrected_content_max_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_child_corrected_content_kind_profiles: parse_kind_profiles(
                    p,
                    "layout_child_corrected_content_kind_profiles",
                ),
                layout_child_roots: p
                    .get("layout_child_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_layout_invalidated_roots: p
                    .get("layout_child_layout_invalidated_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_subtree_dirty_roots: p
                    .get("layout_child_subtree_dirty_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_clean_roots: p
                    .get("layout_child_clean_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_performed_roots: p
                    .get("layout_child_performed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_skipped_roots: p
                    .get("layout_child_skipped_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_bounds_changed_roots: p
                    .get("layout_child_bounds_changed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_bounds_size_changed_roots: p
                    .get("layout_child_bounds_size_changed_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_input_mismatch_roots: p
                    .get("layout_child_input_mismatch_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_input_size_mismatch_roots: p
                    .get("layout_child_input_size_mismatch_roots")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_nodes_visited: p
                    .get("layout_child_nodes_visited")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_nodes_performed: p
                    .get("layout_child_nodes_performed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_kind_profiles: parse_kind_profiles(p, "layout_child_kind_profiles"),
                layout_child_max_us: p
                    .get("layout_child_max_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                layout_child_max_node: p.get("layout_child_max_node").and_then(|v| v.as_u64()),
                layout_child_max_invalidated: p
                    .get("layout_child_max_invalidated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                layout_child_max_subtree_dirty: p
                    .get("layout_child_max_subtree_dirty")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                layout_child_max_subtree_dirty_count: p
                    .get("layout_child_max_subtree_dirty_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64)
                    as u32,
                layout_child_max_nodes_visited: p
                    .get("layout_child_max_nodes_visited")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_max_nodes_performed: p
                    .get("layout_child_max_nodes_performed")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(u32::MAX as u64) as u32,
                layout_child_max_bounds_changed: p
                    .get("layout_child_max_bounds_changed")
                    .and_then(|v| v.as_bool()),
                layout_child_max_bounds_size_changed: p
                    .get("layout_child_max_bounds_size_changed")
                    .and_then(|v| v.as_bool()),
                layout_child_max_input_matches_before: p
                    .get("layout_child_max_input_matches_before")
                    .and_then(|v| v.as_bool()),
                layout_child_max_input_size_matches_before: p
                    .get("layout_child_max_input_size_matches_before")
                    .and_then(|v| v.as_bool()),
                total_us: p.get("total_us").and_then(|v| v.as_u64()).unwrap_or(0),
                element_path: p
                    .get("element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                role: None,
                semantics_test_id: None,
            })
        })
        .collect();

    out.sort_by(|a, b| b.total_us.cmp(&a.total_us));
    out.truncate(max.max(1));

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.semantics_test_id = test_id;
    }

    out
}

pub(super) fn snapshot_widget_measure_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsWidgetMeasureHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("widget_measure_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsWidgetMeasureHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsWidgetMeasureHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            element_path: h
                .get("element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            widget_type: h
                .get("widget_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            measure_time_us: h
                .get("measure_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            inclusive_time_us: h
                .get("inclusive_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn snapshot_command_availability_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsCommandAvailabilityHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("command_availability_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<BundleStatsCommandAvailabilityHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsCommandAvailabilityHotspot {
            command: h
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            route: h
                .get("route")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            start_node: h.get("start_node").and_then(|v| v.as_u64()).unwrap_or(0),
            resolved_node: h.get("resolved_node").and_then(|v| v.as_u64()),
            outcome: h
                .get("outcome")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            elapsed_us: h.get("elapsed_us").and_then(|v| v.as_u64()).unwrap_or(0),
            start_element: h.get("start_element").and_then(|v| v.as_u64()),
            start_element_kind: h
                .get("start_element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            start_element_path: h
                .get("start_element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            resolved_element: h.get("resolved_element").and_then(|v| v.as_u64()),
            resolved_element_kind: h
                .get("resolved_element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            resolved_element_path: h
                .get("resolved_element_path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
        .collect();
    out.sort_by(|a, b| b.elapsed_us.cmp(&a.elapsed_us));
    out.truncate(max.max(1));
    out
}

pub(super) fn snapshot_paint_text_prepare_hotspots(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsPaintTextPrepareHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("paint_text_prepare_hotspots"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if hotspots.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsPaintTextPrepareHotspot> = hotspots
        .iter()
        .take(max.max(1))
        .map(|h| BundleStatsPaintTextPrepareHotspot {
            node: h.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
            element: h.get("element").and_then(|v| v.as_u64()),
            element_kind: h
                .get("element_kind")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            prepare_time_us: h
                .get("prepare_time_us")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            text_len: h
                .get("text_len")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            max_width: h
                .get("max_width")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            wrap: h
                .get("wrap")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            overflow: h
                .get("overflow")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            scale_factor: h
                .get("scale_factor")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32),
            reasons_mask: h
                .get("reasons_mask")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u16::MAX as u64) as u16,
            role: None,
            test_id: None,
        })
        .collect();

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
        item.role = role;
        item.test_id = test_id;
    }

    out
}

pub(super) fn format_text_prepare_reasons(mask: u16) -> String {
    let mut out = String::new();
    let mut push = |name: &str| {
        if !out.is_empty() {
            out.push('|');
        }
        out.push_str(name);
    };
    if mask & (1 << 0) != 0 {
        push("blob");
    }
    if mask & (1 << 1) != 0 {
        push("scale");
    }
    if mask & (1 << 2) != 0 {
        push("text");
    }
    if mask & (1 << 3) != 0 {
        push("rich");
    }
    if mask & (1 << 4) != 0 {
        push("style");
    }
    if mask & (1 << 5) != 0 {
        push("wrap");
    }
    if mask & (1 << 6) != 0 {
        push("overflow");
    }
    if mask & (1 << 7) != 0 {
        push("width");
    }
    if mask & (1 << 8) != 0 {
        push("font");
    }
    if out.is_empty() {
        out.push('0');
    }
    out
}

pub(super) fn snapshot_layout_engine_solves(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsLayoutEngineSolve> {
    let solves = snapshot
        .get("debug")
        .and_then(|v| v.get("layout_engine_solves"))
        .and_then(|v| v.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[]);

    if solves.is_empty() {
        return Vec::new();
    }

    let semantics_index = SemanticsIndex::from_snapshot(semantics, snapshot);

    let mut out: Vec<BundleStatsLayoutEngineSolve> = solves
        .iter()
        .map(|s| {
            let top_measures = s
                .get("top_measures")
                .and_then(|v| v.as_array())
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            let mut top_measures: Vec<BundleStatsLayoutEngineMeasureHotspot> = top_measures
                .iter()
                .take(3)
                .map(|m| {
                    let children = m
                        .get("top_children")
                        .and_then(|v| v.as_array())
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let mut top_children: Vec<BundleStatsLayoutEngineMeasureChildHotspot> =
                        children
                            .iter()
                            .take(3)
                            .map(|c| BundleStatsLayoutEngineMeasureChildHotspot {
                                child: c.get("child").and_then(|v| v.as_u64()).unwrap_or(0),
                                measure_time_us: c
                                    .get("measure_time_us")
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0),
                                calls: c.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                                element: c.get("element").and_then(|v| v.as_u64()),
                                element_kind: c
                                    .get("element_kind")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                role: None,
                                test_id: None,
                            })
                            .collect();

                    for item in &mut top_children {
                        let (role, test_id) =
                            semantics_index.lookup_for_node_or_ancestor_test_id(item.child);
                        item.role = role;
                        item.test_id = test_id;
                    }

                    BundleStatsLayoutEngineMeasureHotspot {
                        node: m.get("node").and_then(|v| v.as_u64()).unwrap_or(0),
                        measure_time_us: m
                            .get("measure_time_us")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                        calls: m.get("calls").and_then(|v| v.as_u64()).unwrap_or(0),
                        cache_hits: m.get("cache_hits").and_then(|v| v.as_u64()).unwrap_or(0),
                        element: m.get("element").and_then(|v| v.as_u64()),
                        element_kind: m
                            .get("element_kind")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        top_children,
                        role: None,
                        test_id: None,
                    }
                })
                .collect();

            for item in &mut top_measures {
                let (role, test_id) =
                    semantics_index.lookup_for_node_or_ancestor_test_id(item.node);
                item.role = role;
                item.test_id = test_id;
            }

            let solve_profile = s.get("solve_profile").and_then(|v| v.as_object()).map(|p| {
                BundleStatsLayoutEngineSolveProfile {
                    reason: p
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    available_w_kind: p
                        .get("available_w_kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    available_h_kind: p
                        .get("available_h_kind")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    available_w: p.get("available_w").and_then(|v| v.as_f64()),
                    available_h: p.get("available_h").and_then(|v| v.as_f64()),
                    previous_available_w_kind: p
                        .get("previous_available_w_kind")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    previous_available_h_kind: p
                        .get("previous_available_h_kind")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    previous_available_w: p.get("previous_available_w").and_then(|v| v.as_f64()),
                    previous_available_h: p.get("previous_available_h").and_then(|v| v.as_f64()),
                    available_w_delta: p.get("available_w_delta").and_then(|v| v.as_f64()),
                    available_h_delta: p.get("available_h_delta").and_then(|v| v.as_f64()),
                    scale_factor: p
                        .get("scale_factor")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0),
                    previous_scale_factor: p.get("previous_scale_factor").and_then(|v| v.as_f64()),
                    scale_factor_delta: p.get("scale_factor_delta").and_then(|v| v.as_f64()),
                    previous_frame_delta: p.get("previous_frame_delta").and_then(|v| v.as_u64()),
                    batch_roots: p.get("batch_roots").and_then(|v| v.as_u64()).unwrap_or(0),
                    subtree_nodes: p.get("subtree_nodes").and_then(|v| v.as_u64()).unwrap_or(0),
                    flex_wrap_patch_time_us: p
                        .get("flex_wrap_patch_time_us")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_visited_nodes: p
                        .get("flex_wrap_patch_visited_nodes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_wrap_nodes: p
                        .get("flex_wrap_patch_wrap_nodes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_candidate_children: p
                        .get("flex_wrap_patch_candidate_children")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_probes: p
                        .get("flex_wrap_patch_probes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_mutations: p
                        .get("flex_wrap_patch_mutations")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    flex_wrap_patch_skipped_no_wrap_descendant: p
                        .get("flex_wrap_patch_skipped_no_wrap_descendant")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                }
            });

            let mut clean_geometry_solve_skip_rejection = s
                .get("clean_geometry_solve_skip_rejection")
                .and_then(|v| v.as_object())
                .map(|r| BundleStatsCleanGeometrySolveSkipRejection {
                    reason: r
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    detail: r
                        .get("detail")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    node: r.get("node").and_then(|v| v.as_u64()),
                    element: r.get("element").and_then(|v| v.as_u64()),
                    element_kind: r
                        .get("element_kind")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    element_path: r
                        .get("element_path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    role: None,
                    test_id: None,
                });

            if let Some(rejection) = clean_geometry_solve_skip_rejection.as_mut()
                && let Some(node) = rejection.node
            {
                let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(node);
                rejection.role = role;
                rejection.test_id = test_id;
            }

            BundleStatsLayoutEngineSolve {
                root_node: s.get("root_node").and_then(|v| v.as_u64()).unwrap_or(0),
                root_element: s.get("root_element").and_then(|v| v.as_u64()),
                root_element_kind: s
                    .get("root_element_kind")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                root_element_path: s
                    .get("root_element_path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                solve_time_us: s.get("solve_time_us").and_then(|v| v.as_u64()).unwrap_or(0),
                solve_profile,
                clean_geometry_solve_skip_rejection,
                measure_calls: s.get("measure_calls").and_then(|v| v.as_u64()).unwrap_or(0),
                measure_cache_hits: s
                    .get("measure_cache_hits")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                measure_time_us: s
                    .get("measure_time_us")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
                top_measures,
                root_role: None,
                root_test_id: None,
            }
        })
        .collect();

    out.sort_by(|a, b| b.solve_time_us.cmp(&a.solve_time_us));
    out.truncate(max);

    for item in &mut out {
        let (role, test_id) = semantics_index.lookup_for_node_or_ancestor_test_id(item.root_node);
        item.root_role = role;
        item.root_test_id = test_id;
    }

    out
}

pub(super) fn snapshot_model_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsModelChangeHotspot {
            model: h.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

pub(super) fn snapshot_model_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsModelChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("model_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsModelChangeUnobserved {
            model: u.get("model").and_then(|v| v.as_u64()).unwrap_or(0),
            created_type: u
                .get("created_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: u
                .get("created_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

pub(super) fn snapshot_global_change_hotspots(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeHotspot> {
    let hotspots = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_hotspots"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    hotspots
        .iter()
        .take(max)
        .map(|h| BundleStatsGlobalChangeHotspot {
            type_name: h
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            observation_edges: h
                .get("observation_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                .min(u32::MAX as u64) as u32,
            changed_at: h
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

pub(super) fn snapshot_global_change_unobserved(
    snapshot: &serde_json::Value,
    max: usize,
) -> Vec<BundleStatsGlobalChangeUnobserved> {
    let unobserved = snapshot
        .get("debug")
        .and_then(|v| v.get("global_change_unobserved"))
        .and_then(|v| v.as_array())
        .map_or(&[][..], |v| v);

    unobserved
        .iter()
        .take(max)
        .map(|u| BundleStatsGlobalChangeUnobserved {
            type_name: u
                .get("type_name")
                .and_then(|v| v.as_str())
                .unwrap_or("?")
                .to_string(),
            changed_at: u
                .get("changed_at")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    let file = m.get("file").and_then(|v| v.as_str())?;
                    let line = m.get("line").and_then(|v| v.as_u64())?;
                    let column = m.get("column").and_then(|v| v.as_u64())?;
                    Some(format!("{}:{}:{}", file, line, column))
                }),
        })
        .collect()
}

pub(super) fn snapshot_lookup_semantics(
    semantics: &crate::json_bundle::SemanticsResolver<'_>,
    snapshot: &serde_json::Value,
    node_id: u64,
) -> (Option<String>, Option<String>) {
    let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

    for n in nodes {
        if n.get("id").and_then(|v| v.as_u64()) == Some(node_id) {
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return (role, test_id);
        }
    }
    (None, None)
}

#[derive(Debug, Clone)]
struct SemanticsNodeLite {
    id: u64,
    parent: Option<u64>,
    role: Option<String>,
    test_id: Option<String>,
}

#[derive(Debug, Default)]
pub(super) struct SemanticsIndex {
    by_id: std::collections::HashMap<u64, SemanticsNodeLite>,
    best_descendant_with_test_id: std::collections::HashMap<u64, (Option<String>, Option<String>)>,
}

impl SemanticsIndex {
    pub(super) fn from_snapshot(
        semantics: &crate::json_bundle::SemanticsResolver<'_>,
        snapshot: &serde_json::Value,
    ) -> Self {
        let nodes = semantics.nodes(snapshot).unwrap_or(&[]);

        let mut by_id: std::collections::HashMap<u64, SemanticsNodeLite> =
            std::collections::HashMap::new();
        by_id.reserve(nodes.len());

        for n in nodes {
            let Some(id) = n.get("id").and_then(|v| v.as_u64()) else {
                continue;
            };

            let parent = n.get("parent").and_then(|v| v.as_u64());
            let role = n
                .get("role")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let test_id = n
                .get("test_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            by_id.insert(
                id,
                SemanticsNodeLite {
                    id,
                    parent,
                    role,
                    test_id,
                },
            );
        }

        let mut best_descendant_with_test_id: std::collections::HashMap<
            u64,
            (Option<String>, Option<String>),
        > = std::collections::HashMap::new();

        for node in by_id.values() {
            let Some(test_id) = node.test_id.as_deref() else {
                continue;
            };
            if test_id.is_empty() {
                continue;
            }

            let mut cursor: Option<u64> = Some(node.id);
            let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    break;
                }

                best_descendant_with_test_id
                    .entry(id)
                    .or_insert_with(|| (node.role.clone(), node.test_id.clone()));

                cursor = by_id.get(&id).and_then(|n| n.parent);
            }
        }

        Self {
            by_id,
            best_descendant_with_test_id,
        }
    }

    pub(super) fn lookup_for_cache_root(&self, root_node: u64) -> (Option<String>, Option<String>) {
        if let Some(node) = self.by_id.get(&root_node) {
            return (node.role.clone(), node.test_id.clone());
        }

        if let Some((role, test_id)) = self.best_descendant_with_test_id.get(&root_node) {
            return (role.clone(), test_id.clone());
        }

        (None, None)
    }

    fn lookup_for_node_or_ancestor_test_id(
        &self,
        node_id: u64,
    ) -> (Option<String>, Option<String>) {
        const MAX_PARENT_HOPS: usize = 16;

        let mut role: Option<String> = None;
        let mut current: Option<u64> = Some(node_id);
        for _ in 0..MAX_PARENT_HOPS {
            let Some(id) = current else {
                break;
            };
            let Some(node) = self.by_id.get(&id) else {
                break;
            };
            if role.is_none() {
                role = node.role.clone();
            }
            if node.test_id.as_ref().is_some_and(|s| !s.is_empty()) {
                return (role, node.test_id.clone());
            }
            current = node.parent;
        }

        (role, None)
    }
}

// NOTE: Gate checks (retained-vlist keep-alive budget, notify hotspot counters, etc.) intentionally
// stay in `crates/fret-diag/src/stats.rs` (or dedicated `*_gates.rs` modules). This file is scoped
// to snapshot-derived helpers used by bundle stats/hotspots reporting.
