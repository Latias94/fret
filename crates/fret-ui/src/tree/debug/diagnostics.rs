#![cfg(feature = "diagnostics")]

use super::super::*;

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugSetChildrenWrite {
    pub parent: NodeId,
    pub frame_id: FrameId,
    pub old_len: u32,
    pub new_len: u32,
    pub old_elements_head: [Option<GlobalElementId>; 4],
    pub new_elements_head: [Option<GlobalElementId>; 4],
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugParentSeverWrite {
    pub child: NodeId,
    pub parent: NodeId,
    pub frame_id: FrameId,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugSetLayerVisibleWrite {
    pub layer: UiLayerId,
    pub frame_id: FrameId,
    pub prev_visible: Option<bool>,
    pub visible: bool,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugOverlayPolicyDecisionWrite {
    pub layer: UiLayerId,
    pub frame_id: FrameId,
    pub kind: &'static str,
    pub present: bool,
    pub interactive: bool,
    pub wants_timer_events: bool,
    pub reason: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugRemoveSubtreeFrameContext {
    pub parent_frame_children_len: Option<u32>,
    pub parent_frame_children_contains_root: Option<bool>,
    pub root_frame_instance_present: bool,
    pub root_frame_children_len: Option<u32>,
    /// Whether this subtree root is reachable from the window's liveness roots when considering
    /// the authoritative retained edges used for GC (ie. the union of `UiTree` and `WindowFrame`
    /// child edges when available).
    pub root_reachable_from_layer_roots: bool,
    pub root_reachable_from_view_cache_roots: Option<bool>,
    pub liveness_layer_roots_len: u32,
    pub view_cache_reuse_roots_len: u32,
    pub view_cache_reuse_root_nodes_len: u32,
    pub trigger_element: Option<GlobalElementId>,
    pub trigger_element_root: Option<GlobalElementId>,
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    pub trigger_element_listed_under_reuse_root: Option<GlobalElementId>,
    pub path_edge_len: u8,
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    pub path_edge_frame_contains_child: [u8; 16],
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiDebugRemoveSubtreeOutcome {
    SkippedLayerRoot,
    RootMissing,
    Removed,
}

#[cfg(feature = "diagnostics")]
#[derive(Debug, Clone, Copy)]
pub struct UiDebugRemoveSubtreeRecord {
    pub outcome: UiDebugRemoveSubtreeOutcome,
    pub frame_id: FrameId,
    pub root: NodeId,
    pub root_element: Option<GlobalElementId>,
    pub root_parent: Option<NodeId>,
    pub root_parent_element: Option<GlobalElementId>,
    pub root_root: Option<NodeId>,
    pub root_layer: Option<UiLayerId>,
    pub root_layer_visible: Option<bool>,
    pub reachable_from_layer_roots: bool,
    pub reachable_from_view_cache_roots: Option<bool>,
    pub unreachable_from_liveness_roots: bool,
    pub liveness_layer_roots_len: Option<u32>,
    pub view_cache_reuse_roots_len: Option<u32>,
    pub view_cache_reuse_root_nodes_len: Option<u32>,
    pub trigger_element: Option<GlobalElementId>,
    pub trigger_element_root: Option<GlobalElementId>,
    pub trigger_element_in_view_cache_keep_alive: Option<bool>,
    pub trigger_element_listed_under_reuse_root: Option<GlobalElementId>,
    pub root_children_len: u32,
    pub root_parent_children_len: Option<u32>,
    pub root_parent_children_contains_root: Option<bool>,
    pub root_parent_frame_children_len: Option<u32>,
    pub root_parent_frame_children_contains_root: Option<bool>,
    pub root_frame_instance_present: Option<bool>,
    pub root_frame_children_len: Option<u32>,
    pub root_path_len: u8,
    pub root_path: [u64; 16],
    pub root_path_truncated: bool,
    pub root_path_edge_len: u8,
    /// For each `root_path` edge (`child -> parent`), whether `UiTree` currently has the
    /// corresponding `parent.children` edge:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing node entry)
    pub root_path_edge_ui_contains_child: [u8; 16],
    /// For each `root_path` edge (`child -> parent`), whether `WindowFrame.children[parent]`
    /// contains the child node:
    /// - `0`: false
    /// - `1`: true
    /// - `2`: unknown (missing frame edge capture)
    pub root_path_edge_frame_contains_child: [u8; 16],
    pub removed_nodes: u32,
    pub removed_head_len: u8,
    pub removed_head: [u64; 16],
    pub removed_tail_len: u8,
    pub removed_tail: [u64; 16],
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
}
