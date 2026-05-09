use super::super::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugScrollHandleChangeKind {
    Layout,
    HitTestOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugScrollAxis {
    X,
    Y,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugScrollLayoutPassKind {
    Probe,
    Final,
}

#[derive(Debug, Clone)]
pub struct UiDebugScrollLayoutProfile {
    pub pass: UiDebugScrollLayoutPassKind,
    pub probe_unbounded: bool,
    pub children: u32,
    pub available: fret_core::Size,
    pub desired: fret_core::Size,
    pub content: fret_core::Size,
    pub post_layout_extents_mode: bool,
    pub interactive_resize: bool,
    pub direct_children_layout_invalidated: bool,
    pub descendant_subtree_layout_dirty: bool,
    pub force_barrier_child_root_relayout: bool,
    pub measure_children_us: u64,
    pub solve_barrier_us: u64,
    pub layout_children_us: u64,
    pub layout_children_first_pass_us: u64,
    pub layout_child_first_pass_nodes_visited: u32,
    pub layout_child_first_pass_nodes_performed: u32,
    pub layout_child_first_pass_max_us: u64,
    pub corrected_content_relayout: bool,
    pub layout_children_corrected_content_us: u64,
    pub layout_child_corrected_content_nodes_visited: u32,
    pub layout_child_corrected_content_nodes_performed: u32,
    pub layout_child_corrected_content_max_us: u64,
    pub layout_child_nodes_visited: u32,
    pub layout_child_nodes_performed: u32,
    pub layout_child_max_us: u64,
    pub layout_child_max_node: Option<NodeId>,
    pub layout_child_max_invalidated: bool,
    pub layout_child_max_subtree_dirty: bool,
    pub layout_child_max_subtree_dirty_count: u32,
    pub layout_child_max_nodes_visited: u32,
    pub layout_child_max_nodes_performed: u32,
    pub layout_child_max_bounds_changed: Option<bool>,
    pub layout_child_max_bounds_size_changed: Option<bool>,
    pub layout_child_max_input_matches_before: Option<bool>,
    pub layout_child_max_input_size_matches_before: Option<bool>,
    pub layout_child_max_bounds_before: Option<Rect>,
    pub layout_child_max_bounds_after: Option<Rect>,
    pub layout_child_max_input_bounds: Rect,
    pub total_us: u64,
    pub element_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiDebugScrollNodeTelemetry {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub test_id: Option<Arc<str>>,
    pub axis: UiDebugScrollAxis,
    pub offset: fret_core::Point,
    pub viewport: fret_core::Size,
    pub content: fret_core::Size,
    pub observed_extent: Option<fret_core::Size>,
    pub overflow_observation: Option<UiDebugScrollOverflowObservationTelemetry>,
    pub layout_profile: Option<UiDebugScrollLayoutProfile>,
}

#[derive(Debug, Clone, Copy)]
pub struct UiDebugScrollOverflowObservationTelemetry {
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

#[derive(Debug, Clone, Copy)]
pub struct UiDebugScrollbarTelemetry {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub axis: UiDebugScrollAxis,
    pub scroll_target: Option<GlobalElementId>,
    pub offset: fret_core::Point,
    pub viewport: fret_core::Size,
    pub content: fret_core::Size,
    pub track: Rect,
    pub thumb: Option<Rect>,
    pub hovered: bool,
    pub dragging: bool,
}

#[derive(Debug, Clone)]
pub struct UiDebugScrollHandleChange {
    pub handle_key: usize,
    pub kind: UiDebugScrollHandleChangeKind,
    pub revision: u64,
    pub prev_revision: Option<u64>,
    pub offset: fret_core::Point,
    pub prev_offset: Option<fret_core::Point>,
    pub viewport: fret_core::Size,
    pub prev_viewport: Option<fret_core::Size>,
    pub content: fret_core::Size,
    pub prev_content: Option<fret_core::Size>,
    pub offset_changed: bool,
    pub viewport_changed: bool,
    pub content_changed: bool,
    pub bound_elements: u32,
    pub bound_nodes_sample: Vec<NodeId>,
    pub upgraded_to_layout_bindings: u32,
}
