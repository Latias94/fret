fn invalidation_source_as_str(source: fret_ui::tree::UiDebugInvalidationSource) -> &'static str {
    match source {
        fret_ui::tree::UiDebugInvalidationSource::ModelChange => "model_change",
        fret_ui::tree::UiDebugInvalidationSource::GlobalChange => "global_change",
        fret_ui::tree::UiDebugInvalidationSource::Notify => "notify",
        fret_ui::tree::UiDebugInvalidationSource::Hover => "hover",
        fret_ui::tree::UiDebugInvalidationSource::Focus => "focus",
        fret_ui::tree::UiDebugInvalidationSource::Other => "other",
    }
}

fn invalidation_detail_as_str(detail: fret_ui::tree::UiDebugInvalidationDetail) -> &'static str {
    detail.as_str().unwrap_or("unknown")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutDirtyDescendantV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    #[serde(default)]
    pub element_path: Option<String>,
    pub subtree_layout_dirty_count: u32,
    #[serde(default)]
    pub source_root_node: Option<u64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub detail: Option<String>,
}

impl UiLayoutDirtyDescendantV1 {
    fn from_record(r: &fret_ui::tree::UiDebugLayoutDirtyDescendant) -> Self {
        Self {
            node: r.node.data().as_ffi(),
            element: r.element.map(|id| id.0),
            element_kind: r.element_kind.map(|s| s.to_string()),
            element_path: r.element_path.clone(),
            subtree_layout_dirty_count: r.subtree_layout_dirty_count,
            source_root_node: r.source_root.map(|id| id.data().as_ffi()),
            source: r.source.map(|s| invalidation_source_as_str(s).to_string()),
            detail: r.detail.map(|d| invalidation_detail_as_str(d).to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutRequestBuildRootV1 {
    pub root_node: u64,
    pub root_kind: String,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_element_kind: Option<String>,
    #[serde(default)]
    pub root_element_path: Option<String>,
    pub elapsed_us: u64,
    pub mode: String,
    pub had_layout_engine_node: bool,
    pub layout_invalidated: bool,
    pub subtree_layout_dirty: bool,
    pub subtree_layout_dirty_count: u32,
    pub descendant_layout_dirty_count: u32,
    pub needs_layout: bool,
    pub is_translation_only: bool,
    pub nodes_marked_seen: u32,
    #[serde(default)]
    pub dirty_descendants: Vec<UiLayoutDirtyDescendantV1>,
}

impl UiLayoutRequestBuildRootV1 {
    fn from_record(r: &fret_ui::tree::UiDebugLayoutRequestBuildRoot) -> Self {
        Self {
            root_node: r.root.data().as_ffi(),
            root_kind: r.root_kind.to_string(),
            root_element: r.root_element.map(|id| id.0),
            root_element_kind: r.root_element_kind.map(|s| s.to_string()),
            root_element_path: r.root_element_path.clone(),
            elapsed_us: r.elapsed.as_micros().min(u64::MAX as u128) as u64,
            mode: r.mode.to_string(),
            had_layout_engine_node: r.had_layout_engine_node,
            layout_invalidated: r.layout_invalidated,
            subtree_layout_dirty: r.subtree_layout_dirty,
            subtree_layout_dirty_count: r.subtree_layout_dirty_count,
            descendant_layout_dirty_count: r.descendant_layout_dirty_count,
            needs_layout: r.needs_layout,
            is_translation_only: r.is_translation_only,
            nodes_marked_seen: r.nodes_marked_seen,
            dirty_descendants: r
                .dirty_descendants
                .iter()
                .map(UiLayoutDirtyDescendantV1::from_record)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineSolveV1 {
    pub root_node: u64,
    #[serde(default)]
    pub root_element: Option<u64>,
    #[serde(default)]
    pub root_element_kind: Option<String>,
    #[serde(default)]
    pub root_element_path: Option<String>,
    pub solve_time_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solve_profile: Option<UiLayoutEngineSolveProfileV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clean_geometry_solve_skip_rejection: Option<UiCleanGeometrySolveSkipRejectionV1>,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    #[serde(default)]
    pub measure_time_us: u64,
    #[serde(default)]
    pub top_measures: Vec<UiLayoutEngineMeasureHotspotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiCleanGeometrySolveSkipRejectionV1 {
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineSolveProfileV1 {
    pub reason: String,
    pub available_w_kind: String,
    pub available_h_kind: String,
    #[serde(default)]
    pub available_w: Option<f32>,
    #[serde(default)]
    pub available_h: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_available_w_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_available_h_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_available_w: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_available_h: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_w_delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub available_h_delta: Option<f32>,
    pub scale_factor: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_scale_factor: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_factor_delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_frame_delta: Option<u64>,
    pub batch_roots: u32,
    pub subtree_nodes: u32,
    #[serde(default)]
    pub flex_wrap_patch_time_us: u64,
    #[serde(default)]
    pub flex_wrap_patch_visited_nodes: u32,
    #[serde(default)]
    pub flex_wrap_patch_wrap_nodes: u32,
    #[serde(default)]
    pub flex_wrap_patch_candidate_children: u32,
    #[serde(default)]
    pub flex_wrap_patch_probes: u32,
    #[serde(default)]
    pub flex_wrap_patch_mutations: u32,
    #[serde(default)]
    pub flex_wrap_patch_skipped_no_wrap_descendant: bool,
}

impl UiLayoutEngineSolveV1 {
    fn from_solve(s: &fret_ui::tree::UiDebugLayoutEngineSolve) -> Self {
        Self {
            root_node: s.root.data().as_ffi(),
            root_element: s.root_element.map(|id| id.0),
            root_element_kind: s.root_element_kind.map(|s| s.to_string()),
            root_element_path: s.root_element_path.clone(),
            solve_time_us: s.solve_time.as_micros().min(u64::MAX as u128) as u64,
            solve_profile: s
                .solve_profile
                .map(|p| UiLayoutEngineSolveProfileV1 {
                    reason: p.reason.to_string(),
                    available_w_kind: p.available_w_kind.to_string(),
                    available_h_kind: p.available_h_kind.to_string(),
                    available_w: p.available_w,
                    available_h: p.available_h,
                    previous_available_w_kind: p.previous_available_w_kind.map(str::to_string),
                    previous_available_h_kind: p.previous_available_h_kind.map(str::to_string),
                    previous_available_w: p.previous_available_w,
                    previous_available_h: p.previous_available_h,
                    available_w_delta: p.available_w_delta,
                    available_h_delta: p.available_h_delta,
                    scale_factor: p.scale_factor,
                    previous_scale_factor: p.previous_scale_factor,
                    scale_factor_delta: p.scale_factor_delta,
                    previous_frame_delta: p.previous_frame_delta,
                    batch_roots: p.batch_roots,
                    subtree_nodes: p.subtree_nodes,
                    flex_wrap_patch_time_us: p
                        .flex_wrap_patch_time
                        .as_micros()
                        .min(u64::MAX as u128) as u64,
                    flex_wrap_patch_visited_nodes: p.flex_wrap_patch_visited_nodes,
                    flex_wrap_patch_wrap_nodes: p.flex_wrap_patch_wrap_nodes,
                    flex_wrap_patch_candidate_children: p.flex_wrap_patch_candidate_children,
                    flex_wrap_patch_probes: p.flex_wrap_patch_probes,
                    flex_wrap_patch_mutations: p.flex_wrap_patch_mutations,
                    flex_wrap_patch_skipped_no_wrap_descendant: p
                        .flex_wrap_patch_skipped_no_wrap_descendant,
                }),
            clean_geometry_solve_skip_rejection: s.clean_geometry_solve_skip_rejection.map(|r| {
                UiCleanGeometrySolveSkipRejectionV1 {
                    reason: r.reason.to_string(),
                    element_kind: r.element_kind.map(str::to_string),
                }
            }),
            measure_calls: s.measure_calls,
            measure_cache_hits: s.measure_cache_hits,
            measure_time_us: s.measure_time.as_micros().min(u64::MAX as u128) as u64,
            top_measures: s
                .top_measures
                .iter()
                .map(|m| UiLayoutEngineMeasureHotspotV1 {
                    node: m.node.data().as_ffi(),
                    measure_time_us: m.measure_time.as_micros().min(u64::MAX as u128) as u64,
                    calls: m.calls,
                    cache_hits: m.cache_hits,
                    element: m.element.map(|id| id.0),
                    element_kind: m.element_kind.map(|s| s.to_string()),
                    top_children: m
                        .top_children
                        .iter()
                        .map(|c| UiLayoutEngineMeasureChildHotspotV1 {
                            child: c.child.data().as_ffi(),
                            measure_time_us: c.measure_time.as_micros().min(u64::MAX as u128)
                                as u64,
                            calls: c.calls,
                            element: c.element.map(|id| id.0),
                            element_kind: c.element_kind.map(|s| s.to_string()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub layout_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

impl UiLayoutHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugLayoutHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            layout_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiWidgetMeasureHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_path: Option<String>,
    pub widget_type: String,
    pub measure_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiCommandAvailabilityHotspotV1 {
    pub command: String,
    pub route: String,
    pub start_node: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_node: Option<u64>,
    pub outcome: String,
    pub elapsed_us: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_element_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_element: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_element_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_element_path: Option<String>,
}

impl UiCommandAvailabilityHotspotV1 {
    pub fn from_hotspot(
        h: &fret_ui::tree::UiDebugCommandAvailabilityHotspot,
        window: AppWindowId,
        element_runtime_state: Option<&ElementRuntime>,
    ) -> Self {
        let start_element_path = h.start_element.and_then(|id| {
            element_runtime_state.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });
        let resolved_element_path = h.resolved_element.and_then(|id| {
            element_runtime_state.and_then(|runtime| runtime.debug_path_for_element(window, id))
        });

        Self {
            command: h.command.as_str().to_string(),
            route: h.route.to_string(),
            start_node: h.start_node.data().as_ffi(),
            resolved_node: h.resolved_node.map(|node| node.data().as_ffi()),
            outcome: match h.outcome {
                fret_ui::CommandAvailability::Available => "available",
                fret_ui::CommandAvailability::Blocked => "blocked",
                fret_ui::CommandAvailability::NotHandled => "not_handled",
            }
            .to_string(),
            elapsed_us: h.elapsed.as_micros() as u64,
            start_element: h.start_element.map(|id| id.0),
            start_element_kind: None,
            start_element_path,
            resolved_element: h.resolved_element.map(|id| id.0),
            resolved_element_kind: None,
            resolved_element_path,
        }
    }
}

impl UiWidgetMeasureHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugWidgetMeasureHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            element_path: h.element_path.clone(),
            widget_type: h.widget_type.to_string(),
            measure_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintWidgetHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub widget_type: String,
    pub paint_time_us: u64,
    #[serde(default)]
    pub inclusive_time_us: u64,
    #[serde(default)]
    pub inclusive_scene_ops_delta: u32,
    #[serde(default)]
    pub exclusive_scene_ops_delta: u32,
}

impl UiPaintWidgetHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintWidgetHotspot) -> Self {
        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: h.element_kind.map(|s| s.to_string()),
            widget_type: h.widget_type.to_string(),
            paint_time_us: h.exclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_time_us: h.inclusive_time.as_micros().min(u64::MAX as u128) as u64,
            inclusive_scene_ops_delta: h.inclusive_scene_ops_delta,
            exclusive_scene_ops_delta: h.exclusive_scene_ops_delta,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPaintTextPrepareHotspotV1 {
    pub node: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    pub prepare_time_us: u64,
    pub text_len: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_factor: Option<f32>,
    #[serde(default)]
    pub reasons_mask: u16,
}

impl UiPaintTextPrepareHotspotV1 {
    fn from_hotspot(h: &fret_ui::tree::UiDebugPaintTextPrepareHotspot) -> Self {
        fn wrap_as_str(wrap: fret_core::TextWrap) -> &'static str {
            match wrap {
                fret_core::TextWrap::None => "none",
                fret_core::TextWrap::Word => "word",
                fret_core::TextWrap::Balance => "balance",
                fret_core::TextWrap::WordBreak => "word_break",
                fret_core::TextWrap::Grapheme => "grapheme",
            }
        }

        fn overflow_as_str(overflow: fret_core::TextOverflow) -> &'static str {
            match overflow {
                fret_core::TextOverflow::Clip => "clip",
                fret_core::TextOverflow::Ellipsis => "ellipsis",
            }
        }

        Self {
            node: h.node.data().as_ffi(),
            element: h.element.map(|id| id.0),
            element_kind: Some(h.element_kind.to_string()),
            prepare_time_us: h.prepare_time.as_micros().min(u64::MAX as u128) as u64,
            text_len: h.text_len,
            max_width: h.constraints.max_width.map(|v| v.0),
            wrap: Some(wrap_as_str(h.constraints.wrap).to_string()),
            overflow: Some(overflow_as_str(h.constraints.overflow).to_string()),
            scale_factor: Some(h.constraints.scale_factor),
            reasons_mask: h.reasons_mask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureHotspotV1 {
    pub node: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    pub cache_hits: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
    #[serde(default)]
    pub top_children: Vec<UiLayoutEngineMeasureChildHotspotV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiLayoutEngineMeasureChildHotspotV1 {
    pub child: u64,
    pub measure_time_us: u64,
    pub calls: u64,
    #[serde(default)]
    pub element: Option<u64>,
    #[serde(default)]
    pub element_kind: Option<String>,
}
