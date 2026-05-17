use super::super::*;

#[derive(Debug, Clone, Default)]
pub struct UiDebugLayoutEngineMeasureHotspot {
    pub node: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub cache_hits: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub top_children: Vec<UiDebugLayoutEngineMeasureChildHotspot>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct UiDebugLayoutEngineMeasureChildHotspot {
    pub child: NodeId,
    pub measure_time: Duration,
    pub calls: u64,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutDirtyDescendant {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub subtree_layout_dirty_count: u32,
    pub source_root: Option<NodeId>,
    pub source: Option<UiDebugInvalidationSource>,
    pub detail: Option<UiDebugInvalidationDetail>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutRequestBuildRoot {
    pub root: NodeId,
    pub root_kind: &'static str,
    pub root_element: Option<GlobalElementId>,
    pub root_element_kind: Option<&'static str>,
    pub root_element_path: Option<String>,
    pub elapsed: Duration,
    pub mode: &'static str,
    pub had_layout_engine_node: bool,
    pub layout_invalidated: bool,
    pub subtree_layout_dirty: bool,
    pub subtree_layout_dirty_count: u32,
    pub descendant_layout_dirty_count: u32,
    pub needs_layout: bool,
    pub is_translation_only: bool,
    pub nodes_marked_seen: u32,
    pub dirty_descendants: Vec<UiDebugLayoutDirtyDescendant>,
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutEngineSolve {
    pub root: NodeId,
    pub root_element: Option<GlobalElementId>,
    pub root_element_kind: Option<&'static str>,
    pub root_element_path: Option<String>,
    pub solve_time: Duration,
    pub solve_profile: Option<UiDebugLayoutEngineSolveProfile>,
    pub clean_geometry_solve_skip_rejection: Option<UiDebugCleanGeometrySolveSkipRejection>,
    pub measure_calls: u64,
    pub measure_cache_hits: u64,
    pub measure_time: Duration,
    pub top_measures: Vec<UiDebugLayoutEngineMeasureHotspot>,
}

#[derive(Debug, Clone)]
pub struct UiDebugCleanGeometrySolveSkipRejection {
    pub reason: &'static str,
    pub node: Option<NodeId>,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugLayoutEngineSolveProfile {
    pub reason: &'static str,
    pub available_w_kind: &'static str,
    pub available_h_kind: &'static str,
    pub available_w: Option<f32>,
    pub available_h: Option<f32>,
    pub previous_available_w_kind: Option<&'static str>,
    pub previous_available_h_kind: Option<&'static str>,
    pub previous_available_w: Option<f32>,
    pub previous_available_h: Option<f32>,
    pub available_w_delta: Option<f32>,
    pub available_h_delta: Option<f32>,
    pub scale_factor: f32,
    pub previous_scale_factor: Option<f32>,
    pub scale_factor_delta: Option<f32>,
    pub previous_frame_delta: Option<u64>,
    pub batch_roots: u32,
    pub subtree_nodes: u32,
    pub flex_wrap_patch_time: Duration,
    pub flex_wrap_patch_visited_nodes: u32,
    pub flex_wrap_patch_wrap_nodes: u32,
    pub flex_wrap_patch_candidate_children: u32,
    pub flex_wrap_patch_probes: u32,
    pub flex_wrap_patch_mutations: u32,
    pub flex_wrap_patch_skipped_no_wrap_descendant: bool,
}

impl From<crate::layout_engine::LayoutEngineSolveProfile> for UiDebugLayoutEngineSolveProfile {
    fn from(profile: crate::layout_engine::LayoutEngineSolveProfile) -> Self {
        Self {
            reason: profile.reason,
            available_w_kind: profile.available_w_kind,
            available_h_kind: profile.available_h_kind,
            available_w: profile.available_w,
            available_h: profile.available_h,
            previous_available_w_kind: profile.previous_available_w_kind,
            previous_available_h_kind: profile.previous_available_h_kind,
            previous_available_w: profile.previous_available_w,
            previous_available_h: profile.previous_available_h,
            available_w_delta: profile.available_w_delta,
            available_h_delta: profile.available_h_delta,
            scale_factor: profile.scale_factor,
            previous_scale_factor: profile.previous_scale_factor,
            scale_factor_delta: profile.scale_factor_delta,
            previous_frame_delta: profile.previous_frame_delta,
            batch_roots: profile.batch_roots,
            subtree_nodes: profile.subtree_nodes,
            flex_wrap_patch_time: profile.flex_wrap_patch.elapsed,
            flex_wrap_patch_visited_nodes: profile.flex_wrap_patch.visited_nodes,
            flex_wrap_patch_wrap_nodes: profile.flex_wrap_patch.wrap_nodes,
            flex_wrap_patch_candidate_children: profile.flex_wrap_patch.candidate_children,
            flex_wrap_patch_probes: profile.flex_wrap_patch.probes,
            flex_wrap_patch_mutations: profile.flex_wrap_patch.mutations,
            flex_wrap_patch_skipped_no_wrap_descendant: profile
                .flex_wrap_patch
                .skipped_no_wrap_descendant,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiDebugLayoutHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugWidgetMeasureHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
}

#[derive(Debug, Clone)]
pub struct UiDebugPaintWidgetHotspot {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub element_kind: Option<&'static str>,
    pub element_path: Option<String>,
    pub widget_type: &'static str,
    pub inclusive_time: Duration,
    pub exclusive_time: Duration,
    pub inclusive_scene_ops_delta: u32,
    pub exclusive_scene_ops_delta: u32,
}
