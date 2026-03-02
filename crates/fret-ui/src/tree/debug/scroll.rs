use super::super::*;

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

#[derive(Debug, Clone, Copy)]
pub struct UiDebugScrollNodeTelemetry {
    pub node: NodeId,
    pub element: Option<GlobalElementId>,
    pub axis: UiDebugScrollAxis,
    pub offset: fret_core::Point,
    pub viewport: fret_core::Size,
    pub content: fret_core::Size,
    pub observed_extent: Option<fret_core::Size>,
    pub overflow_observation: Option<UiDebugScrollOverflowObservationTelemetry>,
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
