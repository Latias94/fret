#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCacheRootStatsV1 {
    pub root: u64,
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub reused: bool,
    #[serde(default = "default_cache_root_layout_dependency")]
    pub layout_dependency: String,
    #[serde(default)]
    pub contained_relayout_in_frame: bool,
    pub paint_replayed_ops: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_child_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtree_nodes_truncated_at: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_in_semantics: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_location: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_old_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_new_len: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head: Vec<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_old_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children_last_set_new_elements_head_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub children_last_set_frame_id: Option<u64>,
    #[serde(default)]
    pub reuse_reason: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct UiCacheRootBoundaryOutcomeV1 {
    build_outcome: &'static str,
    layout_outcome: &'static str,
    paint_outcome: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiBoundaryDiagnosticsV1 {
    pub schema_version: u32,
    pub id: u64,
    pub parent: Option<u64>,
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub kind: String,
    pub source: String,
    pub layout_dependency: String,
    pub layout_definite: bool,
    pub layout_dirty: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_dirty_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_dirty_detail: Option<String>,
    pub prepaint_owner: String,
    #[serde(default)]
    pub hit_test_bounds_owner: String,
    #[serde(default)]
    pub semantics_subtree_owner: String,
    #[serde(default)]
    pub interaction_cache_owner: String,
    pub paint_cache_owner: String,
    pub scene_fragment_owner: String,
    pub scene_fragment_slots: usize,
    pub scene_fragment_entries: usize,
    pub scene_fragment_used_entries: usize,
    pub scene_fragment_rejected_entries: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene_fragment_reject_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reuse_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paint_outcome: Option<String>,
}

impl UiBoundaryDiagnosticsV1 {
    fn from_boundary_stats(
        window: AppWindowId,
        element_runtime: Option<&ElementRuntime>,
        boundary: &fret_ui::tree::UiDebugBoundaryStats,
        cache_root: Option<&UiCacheRootStatsV1>,
        cache_root_outcome: Option<UiCacheRootBoundaryOutcomeV1>,
        max_debug_string_bytes: usize,
    ) -> Self {
        let element_path = boundary
            .element
            .and_then(|id| element_runtime.and_then(|runtime| runtime.debug_path_for_element(window, id)));
        let mut out = Self {
            schema_version: 1,
            id: boundary.id.data().as_ffi(),
            parent: boundary.parent.map(|id| id.data().as_ffi()),
            element: boundary.element.map(|id| id.0),
            element_path,
            kind: boundary.kind.to_string(),
            source: boundary.source.to_string(),
            layout_dependency: boundary.layout_dependency.to_string(),
            layout_definite: boundary.layout_definite,
            layout_dirty: boundary.layout_dirty,
            layout_dirty_source: boundary
                .layout_dirty_source
                .map(|source| invalidation_source_as_str(source).to_string()),
            layout_dirty_detail: boundary
                .layout_dirty_detail
                .and_then(|detail| detail.as_str().map(str::to_string)),
            prepaint_owner: boundary.prepaint_owner.to_string(),
            hit_test_bounds_owner: boundary.hit_test_bounds_owner.to_string(),
            semantics_subtree_owner: boundary.semantics_subtree_owner.to_string(),
            interaction_cache_owner: boundary.interaction_cache_owner.to_string(),
            paint_cache_owner: boundary.paint_cache_owner.to_string(),
            scene_fragment_owner: boundary.scene_fragment_owner.to_string(),
            scene_fragment_slots: boundary.scene_fragment_slots,
            scene_fragment_entries: boundary.scene_fragment_entries,
            scene_fragment_used_entries: boundary.scene_fragment_used_entries,
            scene_fragment_rejected_entries: boundary.scene_fragment_rejected_entries,
            scene_fragment_reject_reason: boundary
                .scene_fragment_reject_reason
                .map(str::to_string),
            build_outcome: cache_root_outcome.map(|outcome| outcome.build_outcome.to_string()),
            reuse_reason: cache_root.and_then(|stats| stats.reuse_reason.clone()),
            layout_outcome: cache_root_outcome.map(|outcome| outcome.layout_outcome.to_string()),
            paint_outcome: cache_root_outcome.map(|outcome| outcome.paint_outcome.to_string()),
        };

        truncate_opt_string_bytes(&mut out.element_path, max_debug_string_bytes);
        truncate_string_bytes(&mut out.kind, max_debug_string_bytes);
        truncate_string_bytes(&mut out.source, max_debug_string_bytes);
        truncate_string_bytes(&mut out.layout_dependency, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.layout_dirty_source, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.layout_dirty_detail, max_debug_string_bytes);
        truncate_string_bytes(&mut out.prepaint_owner, max_debug_string_bytes);
        truncate_string_bytes(&mut out.hit_test_bounds_owner, max_debug_string_bytes);
        truncate_string_bytes(&mut out.semantics_subtree_owner, max_debug_string_bytes);
        truncate_string_bytes(&mut out.interaction_cache_owner, max_debug_string_bytes);
        truncate_string_bytes(&mut out.paint_cache_owner, max_debug_string_bytes);
        truncate_string_bytes(&mut out.scene_fragment_owner, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.scene_fragment_reject_reason, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.build_outcome, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.reuse_reason, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.layout_outcome, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.paint_outcome, max_debug_string_bytes);
        out
    }

    fn cache_root_outcome_from_debug_stats(
        stats: &fret_ui::tree::UiDebugCacheRootStats,
        contained_relayout_in_frame: bool,
    ) -> UiCacheRootBoundaryOutcomeV1 {
        UiCacheRootBoundaryOutcomeV1 {
            build_outcome: cache_root_boundary_build_outcome(stats.reused, stats.reuse_reason),
            layout_outcome: cache_root_boundary_layout_outcome(
                stats.layout_dependency,
                contained_relayout_in_frame,
            ),
            paint_outcome: cache_root_boundary_paint_outcome(stats.paint_replayed_ops),
        }
    }
}

impl UiCacheRootStatsV1 {
    fn from_stats(
        window: AppWindowId,
        ui: &UiTree<App>,
        element_runtime: Option<&ElementRuntime>,
        semantics: Option<&UiSemanticsSnapshotV1>,
        contained_relayout_roots: &HashSet<fret_core::NodeId>,
        stats: &fret_ui::tree::UiDebugCacheRootStats,
        max_debug_string_bytes: usize,
    ) -> Self {
        let element_path = stats.element.and_then(|id| {
            element_runtime.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });

        let direct_child_nodes = ui.children(stats.root).len().min(u32::MAX as usize) as u32;

        // Keep bundles bounded: cache roots can cover large subtrees in real apps.
        const MAX_SUBTREE_NODES: usize = 50_000;
        let mut subtree_nodes_truncated_at: Option<u32> = None;
        let mut seen: HashSet<fret_core::NodeId> = HashSet::new();
        let mut stack: Vec<fret_core::NodeId> = vec![stats.root];
        while let Some(node) = stack.pop() {
            if !seen.insert(node) {
                continue;
            }
            if seen.len() > MAX_SUBTREE_NODES {
                subtree_nodes_truncated_at = Some(MAX_SUBTREE_NODES as u32);
                break;
            }
            for child in ui.children(node) {
                stack.push(child);
            }
        }

        let root_in_semantics = semantics.map(|snap| {
            let id = stats.root.data().as_ffi();
            snap.nodes.iter().any(|n| n.id == id)
        });
        let contained_relayout_in_frame = contained_relayout_roots.contains(&stats.root);
        let layout_dependency = stats.layout_dependency.to_string();

        let (
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
        ) = ui
            .debug_set_children_write_for(stats.root)
            .map(|w| {
                let old_elements_head: Vec<_> = w.old_elements_head.iter().flatten().copied().collect();
                let new_elements_head: Vec<_> = w.new_elements_head.iter().flatten().copied().collect();

                let old_paths: Vec<String> = old_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();
                let new_paths: Vec<String> = new_elements_head
                    .iter()
                    .filter_map(|id| {
                        element_runtime
                            .and_then(|runtime| runtime.debug_path_for_element(window, *id))
                    })
                    .collect();

                (
                    Some(format!("{}:{}:{}", w.file, w.line, w.column)),
                    Some(w.old_len),
                    Some(w.new_len),
                    old_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    new_elements_head.iter().map(|id| id.0).collect::<Vec<_>>(),
                    old_paths,
                    new_paths,
                    Some(w.frame_id.0),
                )
            })
            .unwrap_or((
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                None,
            ));

        let mut out = Self {
            root: stats.root.data().as_ffi(),
            element: stats.element.map(|id| id.0),
            element_path,
            reused: stats.reused,
            layout_dependency,
            contained_relayout_in_frame,
            paint_replayed_ops: stats.paint_replayed_ops,
            direct_child_nodes: Some(direct_child_nodes),
            subtree_nodes: Some(seen.len().min(u32::MAX as usize) as u32),
            subtree_nodes_truncated_at,
            root_in_semantics,
            children_last_set_location,
            children_last_set_old_len,
            children_last_set_new_len,
            children_last_set_old_elements_head,
            children_last_set_new_elements_head,
            children_last_set_old_elements_head_paths,
            children_last_set_new_elements_head_paths,
            children_last_set_frame_id,
            reuse_reason: Some(stats.reuse_reason.as_str().to_string()),
        };

        truncate_opt_string_bytes(&mut out.element_path, max_debug_string_bytes);
        truncate_string_bytes(&mut out.layout_dependency, max_debug_string_bytes);
        truncate_opt_string_bytes(&mut out.children_last_set_location, max_debug_string_bytes);
        truncate_vec_string_bytes(
            &mut out.children_last_set_old_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_vec_string_bytes(
            &mut out.children_last_set_new_elements_head_paths,
            max_debug_string_bytes,
        );
        truncate_opt_string_bytes(&mut out.reuse_reason, max_debug_string_bytes);

        out
    }
}

fn cache_root_boundary_build_outcome(
    reused: bool,
    reason: fret_ui::tree::UiDebugCacheRootReuseReason,
) -> &'static str {
    if reused {
        return "reused";
    }

    match reason {
        fret_ui::tree::UiDebugCacheRootReuseReason::FirstMount => "mounted",
        fret_ui::tree::UiDebugCacheRootReuseReason::NodeRecreated => "node_recreated",
        fret_ui::tree::UiDebugCacheRootReuseReason::ViewCacheDisabled
        | fret_ui::tree::UiDebugCacheRootReuseReason::InspectionActive => "reuse_disabled",
        fret_ui::tree::UiDebugCacheRootReuseReason::ManualCacheRoot => "manual",
        fret_ui::tree::UiDebugCacheRootReuseReason::MarkedReuseRoot
        | fret_ui::tree::UiDebugCacheRootReuseReason::NotMarkedReuseRoot
        | fret_ui::tree::UiDebugCacheRootReuseReason::CacheKeyMismatch
        | fret_ui::tree::UiDebugCacheRootReuseReason::NeedsRerender
        | fret_ui::tree::UiDebugCacheRootReuseReason::LayoutInvalidated => "rebuilt",
    }
}

fn cache_root_boundary_layout_outcome(
    layout_dependency: &str,
    contained_relayout_in_frame: bool,
) -> &'static str {
    if contained_relayout_in_frame {
        "contained_relayout"
    } else if cache_root_layout_dependency_is_contained(layout_dependency) {
        "contained_clean"
    } else {
        "parent_dependent"
    }
}

fn cache_root_layout_dependency_is_contained(layout_dependency: &str) -> bool {
    matches!(layout_dependency, "contained_when_bounds_known")
}

fn default_cache_root_layout_dependency() -> String {
    "parent_dependent".to_string()
}

fn cache_root_boundary_paint_outcome(paint_replayed_ops: u32) -> &'static str {
    if paint_replayed_ops > 0 {
        "scene_ops_replayed"
    } else {
        "not_replayed"
    }
}

#[cfg(test)]
mod cache_root_boundary_tests {
    use super::*;
    use fret_ui::tree::UiDebugCacheRootReuseReason;

    #[test]
    fn cache_root_boundary_build_outcome_tracks_reuse_and_reject_reason() {
        assert_eq!(
            cache_root_boundary_build_outcome(true, UiDebugCacheRootReuseReason::NeedsRerender),
            "reused"
        );
        assert_eq!(
            cache_root_boundary_build_outcome(false, UiDebugCacheRootReuseReason::FirstMount),
            "mounted"
        );
        assert_eq!(
            cache_root_boundary_build_outcome(false, UiDebugCacheRootReuseReason::LayoutInvalidated),
            "rebuilt"
        );
        assert_eq!(
            cache_root_boundary_build_outcome(false, UiDebugCacheRootReuseReason::InspectionActive),
            "reuse_disabled"
        );
    }

    #[test]
    fn cache_root_boundary_layout_outcome_reports_containment_state() {
        assert_eq!(
            cache_root_boundary_layout_outcome("contained_when_bounds_known", true),
            "contained_relayout"
        );
        assert_eq!(
            cache_root_boundary_layout_outcome("contained_when_bounds_known", false),
            "contained_clean"
        );
        assert_eq!(
            cache_root_boundary_layout_outcome("parent_dependent", false),
            "parent_dependent"
        );
    }

    #[test]
    fn cache_root_boundary_paint_outcome_reports_replay_state() {
        assert_eq!(cache_root_boundary_paint_outcome(0), "not_replayed");
        assert_eq!(cache_root_boundary_paint_outcome(1), "scene_ops_replayed");
    }

    #[test]
    fn cache_root_stats_serialization_omits_retired_nested_boundary() {
        let cache_root = UiCacheRootStatsV1 {
            root: 7,
            element: None,
            element_path: None,
            reused: false,
            layout_dependency: "contained_when_bounds_known".to_string(),
            contained_relayout_in_frame: false,
            paint_replayed_ops: 0,
            direct_child_nodes: None,
            subtree_nodes: None,
            subtree_nodes_truncated_at: None,
            root_in_semantics: None,
            children_last_set_location: None,
            children_last_set_old_len: None,
            children_last_set_new_len: None,
            children_last_set_old_elements_head: Vec::new(),
            children_last_set_new_elements_head: Vec::new(),
            children_last_set_old_elements_head_paths: Vec::new(),
            children_last_set_new_elements_head_paths: Vec::new(),
            children_last_set_frame_id: None,
            reuse_reason: Some("needs_rerender".to_string()),
        };

        let value = serde_json::to_value(cache_root).expect("cache root stats json");
        assert!(value.get("boundary").is_none());
        assert_eq!(
            value.get("layout_dependency").and_then(|v| v.as_str()),
            Some("contained_when_bounds_known")
        );
        assert!(value.get("contained_layout").is_none());
    }

    #[test]
    fn boundary_diagnostics_are_built_from_boundary_stats_with_cache_root_outcomes() {
        let node = fret_core::NodeId::from(slotmap::KeyData::from_ffi(7));
        let node_id = node.data().as_ffi();
        let cache_root = UiCacheRootStatsV1 {
            root: node_id,
            element: None,
            element_path: None,
            reused: true,
            layout_dependency: "contained_when_bounds_known".to_string(),
            contained_relayout_in_frame: false,
            paint_replayed_ops: 3,
            direct_child_nodes: None,
            subtree_nodes: None,
            subtree_nodes_truncated_at: None,
            root_in_semantics: None,
            children_last_set_location: None,
            children_last_set_old_len: None,
            children_last_set_new_len: None,
            children_last_set_old_elements_head: Vec::new(),
            children_last_set_new_elements_head: Vec::new(),
            children_last_set_old_elements_head_paths: Vec::new(),
            children_last_set_new_elements_head_paths: Vec::new(),
            children_last_set_frame_id: None,
            reuse_reason: Some("marked_reuse_root".to_string()),
        };

        let boundary_stats = fret_ui::tree::UiDebugBoundaryStats {
            id: node,
            parent: None,
            element: None,
            kind: "view_cache_root",
            source: "view_cache",
            prepaint_owner: "view_boundary_prepaint_state",
            hit_test_bounds_owner: "view_boundary_hit_test_bounds_state",
            semantics_subtree_owner: "view_boundary_semantics_state",
            interaction_cache_owner: "view_boundary_interaction_cache_state",
            paint_cache_owner: "view_boundary_paint_cache_state",
            scene_fragment_owner: "view_boundary_scene_fragment_state",
            scene_fragment_slots: 1,
            scene_fragment_entries: 7,
            scene_fragment_used_entries: 5,
            scene_fragment_rejected_entries: 2,
            scene_fragment_reject_reason: Some("rect_mismatch"),
            layout_dependency: "contained_when_bounds_known",
            layout_definite: true,
            layout_dirty: true,
            layout_dirty_source: Some(fret_ui::tree::UiDebugInvalidationSource::Other),
            layout_dirty_detail: Some(
                fret_ui::tree::UiDebugInvalidationDetail::SubtreeLayoutDirtyRepair,
            ),
        };

        let boundary = UiBoundaryDiagnosticsV1::from_boundary_stats(
            AppWindowId::default(),
            None,
            &boundary_stats,
            Some(&cache_root),
            Some(UiCacheRootBoundaryOutcomeV1 {
                build_outcome: "reused",
                layout_outcome: "contained_clean",
                paint_outcome: "scene_ops_replayed",
            }),
            4096,
        );

        assert_eq!(boundary.id, node_id);
        assert_eq!(boundary.source, "view_cache");
        assert_eq!(boundary.kind, "view_cache_root");
        assert_eq!(boundary.layout_dependency, "contained_when_bounds_known");
        assert!(boundary.layout_definite);
        assert!(boundary.layout_dirty);
        assert_eq!(boundary.layout_dirty_source.as_deref(), Some("other"));
        assert_eq!(
            boundary.layout_dirty_detail.as_deref(),
            Some("subtree_layout_dirty_repair")
        );
        assert_eq!(boundary.prepaint_owner, "view_boundary_prepaint_state");
        assert_eq!(
            boundary.hit_test_bounds_owner,
            "view_boundary_hit_test_bounds_state"
        );
        assert_eq!(
            boundary.semantics_subtree_owner,
            "view_boundary_semantics_state"
        );
        assert_eq!(
            boundary.interaction_cache_owner,
            "view_boundary_interaction_cache_state"
        );
        assert_eq!(
            boundary.paint_cache_owner,
            "view_boundary_paint_cache_state"
        );
        assert_eq!(
            boundary.scene_fragment_owner,
            "view_boundary_scene_fragment_state"
        );
        assert_eq!(boundary.scene_fragment_slots, 1);
        assert_eq!(boundary.scene_fragment_entries, 7);
        assert_eq!(boundary.scene_fragment_used_entries, 5);
        assert_eq!(boundary.scene_fragment_rejected_entries, 2);
        assert_eq!(
            boundary.scene_fragment_reject_reason.as_deref(),
            Some("rect_mismatch")
        );
        assert_eq!(boundary.paint_outcome.as_deref(), Some("scene_ops_replayed"));
        assert_eq!(boundary.build_outcome.as_deref(), Some("reused"));
    }
}
