#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScrollAxisV1 {
    X,
    Y,
    Both,
}

impl UiScrollAxisV1 {
    fn from_axis(axis: fret_ui::tree::UiDebugScrollAxis) -> Self {
        match axis {
            fret_ui::tree::UiDebugScrollAxis::X => Self::X,
            fret_ui::tree::UiDebugScrollAxis::Y => Self::Y,
            fret_ui::tree::UiDebugScrollAxis::Both => Self::Both,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiScrollLayoutPassKindV1 {
    Probe,
    Final,
}

impl UiScrollLayoutPassKindV1 {
    fn from_pass(pass: fret_ui::tree::UiDebugScrollLayoutPassKind) -> Self {
        match pass {
            fret_ui::tree::UiDebugScrollLayoutPassKind::Probe => Self::Probe,
            fret_ui::tree::UiDebugScrollLayoutPassKind::Final => Self::Final,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollLayoutKindProfileV1 {
    pub kind: String,
    pub nodes: u32,
    pub self_us: u64,
    pub total_us: u64,
    pub max_self_us: u64,
    pub max_total_us: u64,
}

impl UiScrollLayoutKindProfileV1 {
    fn from_profile(profile: &fret_ui::tree::UiDebugScrollLayoutKindProfile) -> Self {
        Self {
            kind: profile.kind.to_string(),
            nodes: profile.nodes,
            self_us: profile.self_us,
            total_us: profile.total_us,
            max_self_us: profile.max_self_us,
            max_total_us: profile.max_total_us,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollLayoutPhaseProfileV1 {
    pub phase: String,
    pub us: u64,
}

impl UiScrollLayoutPhaseProfileV1 {
    fn from_profile(profile: &fret_ui::tree::UiDebugScrollLayoutPhaseProfile) -> Self {
        Self {
            phase: profile.phase.to_string(),
            us: profile.us,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollLayoutProfileV1 {
    pub pass: UiScrollLayoutPassKindV1,
    pub probe_unbounded: bool,
    pub children: u32,
    pub available_w: f32,
    pub available_h: f32,
    pub desired_w: f32,
    pub desired_h: f32,
    pub content_w: f32,
    pub content_h: f32,
    pub post_layout_extents_mode: bool,
    pub interactive_resize: bool,
    pub direct_children_layout_invalidated: bool,
    pub descendant_subtree_layout_dirty: bool,
    pub force_barrier_child_root_relayout: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phase_profiles: Vec<UiScrollLayoutPhaseProfileV1>,
    pub measure_children_us: u64,
    pub solve_barrier_us: u64,
    pub layout_children_us: u64,
    #[serde(default)]
    pub layout_children_first_pass_us: u64,
    #[serde(default)]
    pub layout_child_first_pass_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_layout_invalidated_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_subtree_dirty_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_clean_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_performed_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_skipped_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_bounds_changed_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_bounds_size_changed_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_input_mismatch_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_input_size_mismatch_roots: u32,
    #[serde(default)]
    pub layout_child_first_pass_nodes_visited: u32,
    #[serde(default)]
    pub layout_child_first_pass_nodes_performed: u32,
    #[serde(default)]
    pub layout_child_first_pass_max_us: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_child_first_pass_kind_profiles: Vec<UiScrollLayoutKindProfileV1>,
    #[serde(default)]
    pub corrected_content_relayout: bool,
    #[serde(default)]
    pub layout_children_corrected_content_us: u64,
    #[serde(default)]
    pub layout_child_corrected_content_nodes_visited: u32,
    #[serde(default)]
    pub layout_child_corrected_content_nodes_performed: u32,
    #[serde(default)]
    pub layout_child_corrected_content_max_us: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_child_corrected_content_kind_profiles: Vec<UiScrollLayoutKindProfileV1>,
    #[serde(default)]
    pub layout_child_roots: u32,
    #[serde(default)]
    pub layout_child_layout_invalidated_roots: u32,
    #[serde(default)]
    pub layout_child_subtree_dirty_roots: u32,
    #[serde(default)]
    pub layout_child_clean_roots: u32,
    #[serde(default)]
    pub layout_child_performed_roots: u32,
    #[serde(default)]
    pub layout_child_skipped_roots: u32,
    #[serde(default)]
    pub layout_child_bounds_changed_roots: u32,
    #[serde(default)]
    pub layout_child_bounds_size_changed_roots: u32,
    #[serde(default)]
    pub layout_child_input_mismatch_roots: u32,
    #[serde(default)]
    pub layout_child_input_size_mismatch_roots: u32,
    pub layout_child_nodes_visited: u32,
    pub layout_child_nodes_performed: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_child_kind_profiles: Vec<UiScrollLayoutKindProfileV1>,
    pub layout_child_max_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_node: Option<u64>,
    pub layout_child_max_invalidated: bool,
    pub layout_child_max_subtree_dirty: bool,
    pub layout_child_max_subtree_dirty_count: u32,
    pub layout_child_max_nodes_visited: u32,
    pub layout_child_max_nodes_performed: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_bounds_changed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_bounds_size_changed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_input_matches_before: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_input_size_matches_before: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_bounds_before: Option<RectV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_child_max_bounds_after: Option<RectV1>,
    pub layout_child_max_input_bounds: RectV1,
    pub total_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
}

impl UiScrollLayoutProfileV1 {
    fn from_profile(profile: &fret_ui::tree::UiDebugScrollLayoutProfile) -> Self {
        Self {
            pass: UiScrollLayoutPassKindV1::from_pass(profile.pass),
            probe_unbounded: profile.probe_unbounded,
            children: profile.children,
            available_w: profile.available.width.0,
            available_h: profile.available.height.0,
            desired_w: profile.desired.width.0,
            desired_h: profile.desired.height.0,
            content_w: profile.content.width.0,
            content_h: profile.content.height.0,
            post_layout_extents_mode: profile.post_layout_extents_mode,
            interactive_resize: profile.interactive_resize,
            direct_children_layout_invalidated: profile.direct_children_layout_invalidated,
            descendant_subtree_layout_dirty: profile.descendant_subtree_layout_dirty,
            force_barrier_child_root_relayout: profile.force_barrier_child_root_relayout,
            phase_profiles: profile
                .phase_profiles
                .iter()
                .map(UiScrollLayoutPhaseProfileV1::from_profile)
                .collect(),
            measure_children_us: profile.measure_children_us,
            solve_barrier_us: profile.solve_barrier_us,
            layout_children_us: profile.layout_children_us,
            layout_children_first_pass_us: profile.layout_children_first_pass_us,
            layout_child_first_pass_roots: profile.layout_child_first_pass_roots,
            layout_child_first_pass_layout_invalidated_roots: profile
                .layout_child_first_pass_layout_invalidated_roots,
            layout_child_first_pass_subtree_dirty_roots: profile
                .layout_child_first_pass_subtree_dirty_roots,
            layout_child_first_pass_clean_roots: profile.layout_child_first_pass_clean_roots,
            layout_child_first_pass_performed_roots: profile
                .layout_child_first_pass_performed_roots,
            layout_child_first_pass_skipped_roots: profile.layout_child_first_pass_skipped_roots,
            layout_child_first_pass_bounds_changed_roots: profile
                .layout_child_first_pass_bounds_changed_roots,
            layout_child_first_pass_bounds_size_changed_roots: profile
                .layout_child_first_pass_bounds_size_changed_roots,
            layout_child_first_pass_input_mismatch_roots: profile
                .layout_child_first_pass_input_mismatch_roots,
            layout_child_first_pass_input_size_mismatch_roots: profile
                .layout_child_first_pass_input_size_mismatch_roots,
            layout_child_first_pass_nodes_visited: profile.layout_child_first_pass_nodes_visited,
            layout_child_first_pass_nodes_performed:
                profile.layout_child_first_pass_nodes_performed,
            layout_child_first_pass_max_us: profile.layout_child_first_pass_max_us,
            layout_child_first_pass_kind_profiles: profile
                .layout_child_first_pass_kind_profiles
                .iter()
                .map(UiScrollLayoutKindProfileV1::from_profile)
                .collect(),
            corrected_content_relayout: profile.corrected_content_relayout,
            layout_children_corrected_content_us: profile.layout_children_corrected_content_us,
            layout_child_corrected_content_nodes_visited:
                profile.layout_child_corrected_content_nodes_visited,
            layout_child_corrected_content_nodes_performed:
                profile.layout_child_corrected_content_nodes_performed,
            layout_child_corrected_content_max_us: profile.layout_child_corrected_content_max_us,
            layout_child_corrected_content_kind_profiles: profile
                .layout_child_corrected_content_kind_profiles
                .iter()
                .map(UiScrollLayoutKindProfileV1::from_profile)
                .collect(),
            layout_child_roots: profile.layout_child_roots,
            layout_child_layout_invalidated_roots: profile.layout_child_layout_invalidated_roots,
            layout_child_subtree_dirty_roots: profile.layout_child_subtree_dirty_roots,
            layout_child_clean_roots: profile.layout_child_clean_roots,
            layout_child_performed_roots: profile.layout_child_performed_roots,
            layout_child_skipped_roots: profile.layout_child_skipped_roots,
            layout_child_bounds_changed_roots: profile.layout_child_bounds_changed_roots,
            layout_child_bounds_size_changed_roots: profile.layout_child_bounds_size_changed_roots,
            layout_child_input_mismatch_roots: profile.layout_child_input_mismatch_roots,
            layout_child_input_size_mismatch_roots: profile.layout_child_input_size_mismatch_roots,
            layout_child_nodes_visited: profile.layout_child_nodes_visited,
            layout_child_nodes_performed: profile.layout_child_nodes_performed,
            layout_child_kind_profiles: profile
                .layout_child_kind_profiles
                .iter()
                .map(UiScrollLayoutKindProfileV1::from_profile)
                .collect(),
            layout_child_max_us: profile.layout_child_max_us,
            layout_child_max_node: profile.layout_child_max_node.map(key_to_u64),
            layout_child_max_invalidated: profile.layout_child_max_invalidated,
            layout_child_max_subtree_dirty: profile.layout_child_max_subtree_dirty,
            layout_child_max_subtree_dirty_count: profile.layout_child_max_subtree_dirty_count,
            layout_child_max_nodes_visited: profile.layout_child_max_nodes_visited,
            layout_child_max_nodes_performed: profile.layout_child_max_nodes_performed,
            layout_child_max_bounds_changed: profile.layout_child_max_bounds_changed,
            layout_child_max_bounds_size_changed: profile.layout_child_max_bounds_size_changed,
            layout_child_max_input_matches_before: profile.layout_child_max_input_matches_before,
            layout_child_max_input_size_matches_before: profile
                .layout_child_max_input_size_matches_before,
            layout_child_max_bounds_before: profile.layout_child_max_bounds_before.map(RectV1::from),
            layout_child_max_bounds_after: profile.layout_child_max_bounds_after.map(RectV1::from),
            layout_child_max_input_bounds: RectV1::from(profile.layout_child_max_input_bounds),
            total_us: profile.total_us,
            element_path: profile.element_path.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UiScrollOverflowObservationTelemetryV1 {
    pub extent_may_be_stale: bool,
    pub barrier_roots: u8,
    pub wrapper_peel_budget: u8,
    pub wrapper_peeled_max: u8,
    pub wrapper_peel_budget_hit: bool,
    pub immediate_children_visited: u16,
    pub immediate_children_skipped_absolute: u16,
    pub deep_scan_enabled: bool,
    pub deep_scan_budget_nodes: u16,
    pub deep_scan_visited: u16,
    pub deep_scan_budget_hit: bool,
    pub deep_scan_skipped_absolute: u16,
}

impl UiScrollOverflowObservationTelemetryV1 {
    fn from_telemetry(t: fret_ui::tree::UiDebugScrollOverflowObservationTelemetry) -> Self {
        Self {
            extent_may_be_stale: t.extent_may_be_stale,
            barrier_roots: t.barrier_roots,
            wrapper_peel_budget: t.wrapper_peel_budget,
            wrapper_peeled_max: t.wrapper_peeled_max,
            wrapper_peel_budget_hit: t.wrapper_peel_budget_hit,
            immediate_children_visited: t.immediate_children_visited,
            immediate_children_skipped_absolute: t.immediate_children_skipped_absolute,
            deep_scan_enabled: t.deep_scan_enabled,
            deep_scan_budget_nodes: t.deep_scan_budget_nodes,
            deep_scan_visited: t.deep_scan_visited,
            deep_scan_budget_hit: t.deep_scan_budget_hit,
            deep_scan_skipped_absolute: t.deep_scan_skipped_absolute,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiScrollNodeTelemetryV1 {
    pub node: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_id: Option<String>,
    pub axis: UiScrollAxisV1,
    pub offset_x: f32,
    pub offset_y: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub content_w: f32,
    pub content_h: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_h: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow_observation: Option<UiScrollOverflowObservationTelemetryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_profile: Option<UiScrollLayoutProfileV1>,
}

impl UiScrollNodeTelemetryV1 {
    fn from_record(record: &fret_ui::tree::UiDebugScrollNodeTelemetry) -> Self {
        Self {
            node: key_to_u64(record.node),
            element: record.element.map(|e| e.0),
            test_id: record.test_id.as_ref().map(|s| s.as_ref().to_string()),
            axis: UiScrollAxisV1::from_axis(record.axis),
            offset_x: record.offset.x.0,
            offset_y: record.offset.y.0,
            viewport_w: record.viewport.width.0,
            viewport_h: record.viewport.height.0,
            content_w: record.content.width.0,
            content_h: record.content.height.0,
            observed_w: record.observed_extent.map(|s| s.width.0),
            observed_h: record.observed_extent.map(|s| s.height.0),
            overflow_observation: record
                .overflow_observation
                .map(UiScrollOverflowObservationTelemetryV1::from_telemetry),
            layout_profile: record
                .layout_profile
                .as_ref()
                .map(UiScrollLayoutProfileV1::from_profile),
        }
    }
}
