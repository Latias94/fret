//! On-disk wrapper formats and optional helpers.

use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer};

use crate::core::{CanvasRect, CanvasSize, EdgeId, Graph, GraphId, GroupId, NodeId};
pub use crate::interaction::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphModifierKey,
    NodeGraphZoomActivationKey,
};

/// Graph file format version (v1).
pub const GRAPH_FILE_VERSION: u32 = 1;

/// Editor view-state format version (v1).
pub const VIEW_STATE_VERSION: u32 = 2;

/// Default project-scoped view-state path for a graph.
///
/// This follows ADR 0126's recommended `.fret/` layout.
pub fn default_project_view_state_path(graph_id: GraphId) -> PathBuf {
    PathBuf::from(".fret/node_graph/view_state").join(format!("{graph_id}.json"))
}

/// Graph persistence file (v1).
///
/// This wrapper enables stable schema evolution while keeping the inner `Graph` model reusable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphFileV1 {
    /// Graph id (duplicated for quick lookup / validation).
    pub graph_id: GraphId,
    /// File wrapper version.
    pub graph_version: u32,
    /// Graph document.
    pub graph: Graph,
}

impl GraphFileV1 {
    /// Wraps a graph into a v1 file object.
    pub fn from_graph(graph: Graph) -> Self {
        Self {
            graph_id: graph.graph_id,
            graph_version: GRAPH_FILE_VERSION,
            graph,
        }
    }

    /// Validates wrapper invariants.
    pub fn validate(&self) -> Result<(), GraphFileError> {
        if self.graph_id != self.graph.graph_id {
            return Err(GraphFileError::InconsistentGraphId);
        }
        Ok(())
    }

    /// Loads a JSON file.
    ///
    /// Backward compatibility: accepts both the wrapped form and a plain `Graph` root object.
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, GraphFileError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| GraphFileError::Read {
            path: path.display().to_string(),
            source,
        })?;

        match serde_json::from_slice::<Self>(&bytes) {
            Ok(v) => {
                v.validate()?;
                Ok(v)
            }
            Err(new_err) => match serde_json::from_slice::<Graph>(&bytes) {
                Ok(graph) => Ok(Self::from_graph(graph)),
                Err(_old_err) => Err(GraphFileError::Parse {
                    path: path.display().to_string(),
                    source: new_err,
                }),
            },
        }
    }

    /// Loads the JSON file if it exists.
    pub fn load_json_if_exists(path: impl AsRef<Path>) -> Result<Option<Self>, GraphFileError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json(path).map(Some)
    }

    /// Saves the JSON file (pretty-printed).
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), GraphFileError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| GraphFileError::Write {
                path: path.display().to_string(),
                source,
            })?;
        }
        let bytes =
            serde_json::to_vec_pretty(self).map_err(|source| GraphFileError::Serialize {
                path: path.display().to_string(),
                source,
            })?;
        std::fs::write(path, bytes).map_err(|source| GraphFileError::Write {
            path: path.display().to_string(),
            source,
        })
    }
}

/// Errors for reading/writing graph files.
#[derive(Debug, thiserror::Error)]
pub enum GraphFileError {
    /// Read failure.
    #[error("failed to read graph file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    /// JSON parse failure.
    #[error("failed to parse graph file JSON: {path}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    /// Write failure.
    #[error("failed to write graph file: {path}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    /// JSON serialization failure.
    #[error("failed to serialize graph file JSON: {path}")]
    Serialize {
        path: String,
        source: serde_json::Error,
    },
    /// Wrapper id mismatch.
    #[error("graph file wrapper graph_id does not match graph.graph_id")]
    InconsistentGraphId,
}

/// Pure persisted view-state payload.
///
/// This excludes interaction policy and runtime tuning so persistence boundaries can evolve without
/// forcing every in-memory/runtime consumer to change in the same step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphPureViewState {
    #[serde(default)]
    pub pan: crate::core::CanvasPoint,
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_nodes: Vec<NodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_edges: Vec<EdgeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_groups: Vec<GroupId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub draw_order: Vec<NodeId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_draw_order: Vec<GroupId>,
}

impl Default for NodeGraphPureViewState {
    fn default() -> Self {
        Self {
            pan: crate::core::CanvasPoint::default(),
            zoom: default_zoom(),
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
        }
    }
}

/// Node graph editor view-state.
///
/// This is intentionally separate from graph semantics and may be stored per-user/per-project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphViewState {
    /// Canvas pan in graph space.
    #[serde(default)]
    pub pan: crate::core::CanvasPoint,
    /// Zoom factor.
    #[serde(default = "default_zoom")]
    pub zoom: f32,
    /// Selected nodes (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_nodes: Vec<NodeId>,
    /// Selected edges (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_edges: Vec<EdgeId>,
    /// Selected groups (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_groups: Vec<GroupId>,
    /// Explicit draw order (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub draw_order: Vec<NodeId>,
    /// Explicit group draw order (optional).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub group_draw_order: Vec<GroupId>,
}

impl Default for NodeGraphViewState {
    fn default() -> Self {
        Self {
            pan: crate::core::CanvasPoint::default(),
            zoom: default_zoom(),
            selected_nodes: Vec::new(),
            selected_edges: Vec::new(),
            selected_groups: Vec::new(),
            draw_order: Vec::new(),
            group_draw_order: Vec::new(),
        }
    }
}

impl NodeGraphViewState {
    /// Removes stale IDs (selection / draw order) that no longer exist in the target graph.
    pub fn sanitize_for_graph(&mut self, graph: &Graph) {
        let visible_node = |id: &NodeId| graph.nodes.get(id).is_some_and(|n| !n.hidden);

        self.selected_nodes.retain(visible_node);
        self.selected_edges.retain(|id| {
            let Some(edge) = graph.edges.get(id) else {
                return false;
            };
            let Some(from) = graph.ports.get(&edge.from) else {
                return false;
            };
            let Some(to) = graph.ports.get(&edge.to) else {
                return false;
            };
            visible_node(&from.node) && visible_node(&to.node)
        });
        self.selected_groups
            .retain(|id| graph.groups.contains_key(id));
        self.draw_order.retain(visible_node);
        self.group_draw_order
            .retain(|id| graph.groups.contains_key(id));
    }
}

impl From<NodeGraphPureViewState> for NodeGraphViewState {
    fn from(value: NodeGraphPureViewState) -> Self {
        Self {
            pan: value.pan,
            zoom: value.zoom,
            selected_nodes: value.selected_nodes,
            selected_edges: value.selected_edges,
            selected_groups: value.selected_groups,
            draw_order: value.draw_order,
            group_draw_order: value.group_draw_order,
        }
    }
}

impl From<NodeGraphViewState> for NodeGraphPureViewState {
    fn from(value: NodeGraphViewState) -> Self {
        Self {
            pan: value.pan,
            zoom: value.zoom,
            selected_nodes: value.selected_nodes,
            selected_edges: value.selected_edges,
            selected_groups: value.selected_groups,
            draw_order: value.draw_order,
            group_draw_order: value.group_draw_order,
        }
    }
}

impl From<&NodeGraphViewState> for NodeGraphPureViewState {
    fn from(value: &NodeGraphViewState) -> Self {
        Self {
            pan: value.pan,
            zoom: value.zoom,
            selected_nodes: value.selected_nodes.clone(),
            selected_edges: value.selected_edges.clone(),
            selected_groups: value.selected_groups.clone(),
            draw_order: value.draw_order.clone(),
            group_draw_order: value.group_draw_order.clone(),
        }
    }
}

fn default_zoom() -> f32 {
    1.0
}

/// Behavior for selecting edges during marquee (box) selection.
///
/// XyFlow selects edges connected to the selected nodes.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphBoxSelectEdges {
    /// Do not select edges from a marquee selection.
    None,
    /// Select edges connected to any selected node (XyFlow default).
    #[default]
    Connected,
    /// Select edges only when both endpoints are within the marquee-selected node set.
    BothEndpoints,
}

/// Behavior for selecting nodes during marquee (box) selection.
///
/// This matches XyFlow's `selectionMode`:
/// - `full`: select nodes only when their rect is fully contained in the marquee.
/// - `partial`: select nodes when they intersect the marquee.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphSelectionMode {
    /// Select nodes only when fully contained by the marquee (XyFlow default).
    #[default]
    Full,
    /// Select nodes when partially intersecting the marquee.
    Partial,
}

/// Node origin (anchor) used to interpret `Node.pos` (XyFlow `nodeOrigin`).
///
/// This is expressed as a normalized fraction of the node rect:
/// - `(0.0, 0.0)` means `Node.pos` is the node's top-left.
/// - `(0.5, 0.5)` means `Node.pos` is the node's center.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphNodeOrigin {
    pub x: f32,
    pub y: f32,
}

impl NodeGraphNodeOrigin {
    pub fn normalized(self) -> Self {
        let mut out = self;
        if !out.x.is_finite() {
            out.x = 0.0;
        }
        if !out.y.is_finite() {
            out.y = 0.0;
        }
        out.x = out.x.clamp(0.0, 1.0);
        out.y = out.y.clamp(0.0, 1.0);
        out
    }
}

impl<'de> Deserialize<'de> for NodeGraphBoxSelectEdges {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = NodeGraphBoxSelectEdges;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a bool or one of: none, connected, both_endpoints")
            }

            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(if v {
                    NodeGraphBoxSelectEdges::Connected
                } else {
                    NodeGraphBoxSelectEdges::None
                })
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                match v {
                    "none" => Ok(NodeGraphBoxSelectEdges::None),
                    "connected" => Ok(NodeGraphBoxSelectEdges::Connected),
                    "both_endpoints" => Ok(NodeGraphBoxSelectEdges::BothEndpoints),
                    other => Err(E::custom(format!(
                        "unrecognized box select edges mode: {other}"
                    ))),
                }
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Serialized key code (a `keyboard_types::Code`), stored as a string like `"Space"` or `"KeyA"`.
///
/// This is intentionally aligned with the `KeyboardEvent.code` naming used by XyFlow for
/// `panActivationKeyCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeGraphKeyCode(pub fret_core::KeyCode);

impl Serialize for NodeGraphKeyCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for NodeGraphKeyCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let code = fret_core::KeyCode::from_str(&s)
            .map_err(|_| serde::de::Error::custom(format!("unrecognized key code: {s}")))?;
        Ok(Self(code))
    }
}

/// Delete key binding for removing the current selection (XyFlow `deleteKeyCode`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphDeleteKey {
    /// Delete is disabled.
    None,
    /// Use Backspace (XyFlow default).
    #[default]
    Backspace,
    /// Use Delete.
    Delete,
    /// Accept either Backspace or Delete.
    BackspaceOrDelete,
}

impl NodeGraphDeleteKey {
    pub fn matches(self, key: fret_core::KeyCode) -> bool {
        use fret_core::KeyCode;

        match self {
            Self::None => false,
            Self::Backspace => key == KeyCode::Backspace,
            Self::Delete => key == KeyCode::Delete,
            Self::BackspaceOrDelete => matches!(key, KeyCode::Backspace | KeyCode::Delete),
        }
    }
}

/// Nudge step semantics for keyboard-driven movement.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphNudgeStepMode {
    /// Interprets the step as screen-space pixels (converted to canvas units by dividing by zoom).
    #[default]
    ScreenPx,
    /// Uses the editor snap grid (`snap_grid`) as the step (canvas-space).
    Grid,
}

/// Auto-pan tuning for drag/connect/focus workflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphAutoPanTuning {
    #[serde(default)]
    pub on_node_drag: bool,
    #[serde(default)]
    pub on_connect: bool,
    #[serde(default)]
    pub on_node_focus: bool,

    /// Speed in screen pixels per second (approximate).
    #[serde(default = "default_auto_pan_speed")]
    pub speed: f32,

    /// Margin from viewport edge in screen pixels that triggers auto-pan.
    #[serde(default = "default_auto_pan_margin")]
    pub margin: f32,
}

fn default_auto_pan_speed() -> f32 {
    900.0
}

fn default_auto_pan_margin() -> f32 {
    24.0
}

impl Default for NodeGraphAutoPanTuning {
    fn default() -> Self {
        Self {
            on_node_drag: true,
            on_connect: true,
            on_node_focus: false,
            speed: default_auto_pan_speed(),
            margin: default_auto_pan_margin(),
        }
    }
}

/// Momentum configuration for canvas panning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphPanInertiaTuning {
    /// Enables inertial panning after releasing the pan gesture.
    #[serde(default)]
    pub enabled: bool,

    /// Exponential damping factor applied to velocity (1 / seconds).
    #[serde(default = "default_pan_inertia_decay_per_s")]
    pub decay_per_s: f32,

    /// Minimum screen speed (px/s) required to keep inertia running.
    #[serde(default = "default_pan_inertia_min_speed")]
    pub min_speed: f32,

    /// Maximum screen speed (px/s) at inertia start (clamp).
    #[serde(default = "default_pan_inertia_max_speed")]
    pub max_speed: f32,
}

fn default_pan_inertia_decay_per_s() -> f32 {
    14.0
}

fn default_pan_inertia_min_speed() -> f32 {
    36.0
}

fn default_pan_inertia_max_speed() -> f32 {
    8000.0
}

impl Default for NodeGraphPanInertiaTuning {
    fn default() -> Self {
        Self {
            enabled: false,
            decay_per_s: default_pan_inertia_decay_per_s(),
            min_speed: default_pan_inertia_min_speed(),
            max_speed: default_pan_inertia_max_speed(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGraphPanOnDragButtons {
    /// Pan the canvas by dragging on empty background with the left mouse button.
    #[serde(default)]
    pub left: bool,
    /// Pan the canvas by dragging with the middle mouse button.
    #[serde(default)]
    pub middle: bool,
    /// Pan the canvas by dragging with the right mouse button.
    ///
    /// When enabled, apps should provide an alternate way to open context menus (or make context
    /// menus conditional on "click without pan"), matching XyFlow's `panOnDrag={[2]}` patterns.
    #[serde(default)]
    pub right: bool,
}

fn default_pan_on_drag_buttons() -> NodeGraphPanOnDragButtons {
    NodeGraphPanOnDragButtons {
        left: true,
        middle: true,
        right: false,
    }
}

fn default_box_select_edges() -> NodeGraphBoxSelectEdges {
    NodeGraphBoxSelectEdges::Connected
}

fn default_nodes_draggable() -> bool {
    true
}

fn default_nodes_connectable() -> bool {
    true
}

fn default_nodes_deletable() -> bool {
    true
}

fn default_edges_deletable() -> bool {
    true
}

fn default_bezier_hit_test_steps() -> u8 {
    24
}

fn default_spatial_index_tuning() -> NodeGraphSpatialIndexTuning {
    NodeGraphSpatialIndexTuning::default()
}

fn default_paint_cache_prune_tuning() -> NodeGraphPaintCachePruneTuning {
    NodeGraphPaintCachePruneTuning::default()
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphPanOnScrollMode {
    #[default]
    Free,
    Horizontal,
    Vertical,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphViewportInterpolate {
    Linear,
    #[default]
    Smooth,
}

/// Easing curve for animated viewport changes (XyFlow `fitViewOptions.ease`).
///
/// Note: this is an optional override. When unset, the legacy behavior is derived from
/// `frame_view_interpolate` for backward compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphViewportEase {
    Linear,
    /// Smoothstep `t*t*(3-2*t)` (close to common editor defaults).
    Smoothstep,
    /// Cubic ease-in-out.
    CubicInOut,
}

/// Persisted runtime-heavy tuning for the node graph editor.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphRuntimeTuning {
    #[serde(default = "default_spatial_index_tuning")]
    pub spatial_index: NodeGraphSpatialIndexTuning,
    #[serde(default = "default_only_render_visible_elements")]
    pub only_render_visible_elements: bool,
    #[serde(default = "default_paint_cache_prune_tuning")]
    pub paint_cache_prune: NodeGraphPaintCachePruneTuning,
}

impl NodeGraphRuntimeTuning {
    pub fn is_default(this: &Self) -> bool {
        this == &Self::default()
    }
}

impl Default for NodeGraphRuntimeTuning {
    fn default() -> Self {
        Self {
            spatial_index: default_spatial_index_tuning(),
            only_render_visible_elements: default_only_render_visible_elements(),
            paint_cache_prune: default_paint_cache_prune_tuning(),
        }
    }
}

/// Persisted interaction configuration stored alongside view state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphInteractionConfig {
    #[serde(default = "default_elements_selectable")]
    pub elements_selectable: bool,
    #[serde(default = "default_nodes_draggable")]
    pub nodes_draggable: bool,
    #[serde(default = "default_nodes_connectable")]
    pub nodes_connectable: bool,
    #[serde(default = "default_nodes_deletable")]
    pub nodes_deletable: bool,
    #[serde(default = "default_edges_selectable")]
    pub edges_selectable: bool,
    #[serde(default = "default_edges_deletable")]
    pub edges_deletable: bool,
    #[serde(default = "default_edges_focusable")]
    pub edges_focusable: bool,
    #[serde(default = "default_edges_reconnectable")]
    pub edges_reconnectable: bool,
    #[serde(default)]
    pub connection_mode: NodeGraphConnectionMode,
    #[serde(default = "default_connection_radius")]
    pub connection_radius: f32,
    #[serde(default = "default_reconnect_radius")]
    pub reconnect_radius: f32,
    #[serde(default)]
    pub reconnect_on_drop_empty: bool,
    #[serde(default = "default_edge_interaction_width")]
    pub edge_interaction_width: f32,
    #[serde(default = "default_bezier_hit_test_steps")]
    pub bezier_hit_test_steps: u8,
    #[serde(default = "default_elevate_nodes_on_select")]
    pub elevate_nodes_on_select: bool,
    #[serde(default = "default_elevate_edges_on_select")]
    pub elevate_edges_on_select: bool,
    #[serde(default)]
    pub snap_to_grid: bool,
    #[serde(default = "default_snap_grid")]
    pub snap_grid: CanvasSize,
    #[serde(default = "default_snaplines")]
    pub snaplines: bool,
    #[serde(default = "default_snaplines_threshold")]
    pub snaplines_threshold: f32,
    #[serde(default = "default_pan_on_scroll")]
    pub pan_on_scroll: bool,
    #[serde(default = "default_pan_on_drag_buttons")]
    pub pan_on_drag: NodeGraphPanOnDragButtons,
    #[serde(default)]
    pub selection_on_drag: bool,
    #[serde(default)]
    pub selection_mode: NodeGraphSelectionMode,
    #[serde(
        default = "default_box_select_edges",
        alias = "box_select_connected_edges"
    )]
    pub box_select_edges: NodeGraphBoxSelectEdges,
    #[serde(default = "default_selection_key")]
    pub selection_key: NodeGraphModifierKey,
    #[serde(default = "default_multi_selection_key")]
    pub multi_selection_key: NodeGraphModifierKey,
    #[serde(default)]
    pub delete_key: NodeGraphDeleteKey,
    #[serde(default)]
    pub nudge_step_mode: NodeGraphNudgeStepMode,
    #[serde(default = "default_nudge_step_px")]
    pub nudge_step_px: f32,
    #[serde(default = "default_nudge_fast_step_px")]
    pub nudge_fast_step_px: f32,
    #[serde(default)]
    pub disable_keyboard_a11y: bool,
    #[serde(default = "default_pane_click_distance")]
    pub pane_click_distance: f32,
    #[serde(
        default = "default_pan_activation_key_code",
        skip_serializing_if = "Option::is_none"
    )]
    pub pan_activation_key_code: Option<NodeGraphKeyCode>,
    #[serde(default = "default_space_to_pan")]
    pub space_to_pan: bool,
    #[serde(default = "default_pan_on_scroll_speed")]
    pub pan_on_scroll_speed: f32,
    #[serde(default)]
    pub pan_on_scroll_mode: NodeGraphPanOnScrollMode,
    #[serde(default)]
    pub pan_inertia: NodeGraphPanInertiaTuning,
    #[serde(default = "default_zoom_on_scroll")]
    pub zoom_on_scroll: bool,
    #[serde(default = "default_zoom_on_scroll_speed")]
    pub zoom_on_scroll_speed: f32,
    #[serde(default = "default_zoom_on_pinch")]
    pub zoom_on_pinch: bool,
    #[serde(default = "default_zoom_on_pinch_speed")]
    pub zoom_on_pinch_speed: f32,
    #[serde(default = "default_zoom_on_double_click")]
    pub zoom_on_double_click: bool,
    #[serde(default = "default_frame_view_duration_ms")]
    pub frame_view_duration_ms: u32,
    #[serde(default)]
    pub frame_view_interpolate: NodeGraphViewportInterpolate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_view_ease: Option<NodeGraphViewportEase>,
    #[serde(default = "default_frame_view_padding")]
    pub frame_view_padding: f32,
    #[serde(default = "default_reroute_on_edge_double_click")]
    pub reroute_on_edge_double_click: bool,
    #[serde(default = "default_edge_insert_on_alt_drag")]
    pub edge_insert_on_alt_drag: bool,
    #[serde(default)]
    pub zoom_activation_key: NodeGraphZoomActivationKey,
    #[serde(default = "default_node_drag_threshold")]
    pub node_drag_threshold: f32,
    #[serde(default)]
    pub node_drag_handle_mode: NodeGraphDragHandleMode,
    #[serde(default = "default_node_click_distance")]
    pub node_click_distance: f32,
    #[serde(default = "default_connection_drag_threshold")]
    pub connection_drag_threshold: f32,
    #[serde(default)]
    pub connect_on_click: bool,
    #[serde(default)]
    pub auto_pan: NodeGraphAutoPanTuning,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translate_extent: Option<CanvasRect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_extent: Option<CanvasRect>,
    #[serde(default)]
    pub node_origin: NodeGraphNodeOrigin,
}

impl NodeGraphInteractionConfig {
    pub fn is_default(this: &Self) -> bool {
        this == &Self::default()
    }
}

impl Default for NodeGraphInteractionConfig {
    fn default() -> Self {
        NodeGraphInteractionState::default().config()
    }
}

/// Persisted editor configuration stored alongside pure view state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphEditorConfig {
    #[serde(
        default,
        skip_serializing_if = "NodeGraphInteractionConfig::is_default"
    )]
    pub interaction: NodeGraphInteractionConfig,
    #[serde(default, skip_serializing_if = "NodeGraphRuntimeTuning::is_default")]
    pub runtime_tuning: NodeGraphRuntimeTuning,
}

impl NodeGraphEditorConfig {
    pub fn is_default(this: &Self) -> bool {
        this == &Self::default()
    }

    pub fn resolved_interaction_state(&self) -> NodeGraphInteractionState {
        NodeGraphInteractionState::from_parts(&self.interaction, &self.runtime_tuning)
    }
}

impl Default for NodeGraphEditorConfig {
    fn default() -> Self {
        Self {
            interaction: NodeGraphInteractionConfig::default(),
            runtime_tuning: NodeGraphRuntimeTuning::default(),
        }
    }
}

/// Resolved runtime interaction state assembled from persisted config and runtime tuning.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphInteractionState {
    pub elements_selectable: bool,
    pub nodes_draggable: bool,
    pub nodes_connectable: bool,
    pub nodes_deletable: bool,
    pub edges_selectable: bool,
    pub edges_deletable: bool,
    pub edges_focusable: bool,
    pub edges_reconnectable: bool,
    pub connection_mode: NodeGraphConnectionMode,
    pub connection_radius: f32,
    pub reconnect_radius: f32,
    pub reconnect_on_drop_empty: bool,
    pub edge_interaction_width: f32,
    pub bezier_hit_test_steps: u8,
    pub spatial_index: NodeGraphSpatialIndexTuning,
    pub only_render_visible_elements: bool,
    pub elevate_nodes_on_select: bool,
    pub elevate_edges_on_select: bool,
    pub paint_cache_prune: NodeGraphPaintCachePruneTuning,
    pub snap_to_grid: bool,
    pub snap_grid: CanvasSize,
    pub snaplines: bool,
    pub snaplines_threshold: f32,
    pub pan_on_scroll: bool,
    pub pan_on_drag: NodeGraphPanOnDragButtons,
    pub selection_on_drag: bool,
    pub selection_mode: NodeGraphSelectionMode,
    pub box_select_edges: NodeGraphBoxSelectEdges,
    pub selection_key: NodeGraphModifierKey,
    pub multi_selection_key: NodeGraphModifierKey,
    pub delete_key: NodeGraphDeleteKey,
    pub nudge_step_mode: NodeGraphNudgeStepMode,
    pub nudge_step_px: f32,
    pub nudge_fast_step_px: f32,
    pub disable_keyboard_a11y: bool,
    pub pane_click_distance: f32,
    pub pan_activation_key_code: Option<NodeGraphKeyCode>,
    pub space_to_pan: bool,
    pub pan_on_scroll_speed: f32,
    pub pan_on_scroll_mode: NodeGraphPanOnScrollMode,
    pub pan_inertia: NodeGraphPanInertiaTuning,
    pub zoom_on_scroll: bool,
    pub zoom_on_scroll_speed: f32,
    pub zoom_on_pinch: bool,
    pub zoom_on_pinch_speed: f32,
    pub zoom_on_double_click: bool,
    pub frame_view_duration_ms: u32,
    pub frame_view_interpolate: NodeGraphViewportInterpolate,
    pub frame_view_ease: Option<NodeGraphViewportEase>,
    pub frame_view_padding: f32,
    pub reroute_on_edge_double_click: bool,
    pub edge_insert_on_alt_drag: bool,
    pub zoom_activation_key: NodeGraphZoomActivationKey,
    pub node_drag_threshold: f32,
    pub node_drag_handle_mode: NodeGraphDragHandleMode,
    pub node_click_distance: f32,
    pub connection_drag_threshold: f32,
    pub connect_on_click: bool,
    pub auto_pan: NodeGraphAutoPanTuning,
    pub translate_extent: Option<CanvasRect>,
    pub node_extent: Option<CanvasRect>,
    pub node_origin: NodeGraphNodeOrigin,
}

impl NodeGraphInteractionState {
    pub fn from_parts(
        config: &NodeGraphInteractionConfig,
        runtime_tuning: &NodeGraphRuntimeTuning,
    ) -> Self {
        Self {
            elements_selectable: config.elements_selectable,
            nodes_draggable: config.nodes_draggable,
            nodes_connectable: config.nodes_connectable,
            nodes_deletable: config.nodes_deletable,
            edges_selectable: config.edges_selectable,
            edges_deletable: config.edges_deletable,
            edges_focusable: config.edges_focusable,
            edges_reconnectable: config.edges_reconnectable,
            connection_mode: config.connection_mode,
            connection_radius: config.connection_radius,
            reconnect_radius: config.reconnect_radius,
            reconnect_on_drop_empty: config.reconnect_on_drop_empty,
            edge_interaction_width: config.edge_interaction_width,
            bezier_hit_test_steps: config.bezier_hit_test_steps,
            spatial_index: runtime_tuning.spatial_index,
            only_render_visible_elements: runtime_tuning.only_render_visible_elements,
            elevate_nodes_on_select: config.elevate_nodes_on_select,
            elevate_edges_on_select: config.elevate_edges_on_select,
            paint_cache_prune: runtime_tuning.paint_cache_prune,
            snap_to_grid: config.snap_to_grid,
            snap_grid: config.snap_grid,
            snaplines: config.snaplines,
            snaplines_threshold: config.snaplines_threshold,
            pan_on_scroll: config.pan_on_scroll,
            pan_on_drag: config.pan_on_drag,
            selection_on_drag: config.selection_on_drag,
            selection_mode: config.selection_mode,
            box_select_edges: config.box_select_edges,
            selection_key: config.selection_key,
            multi_selection_key: config.multi_selection_key,
            delete_key: config.delete_key,
            nudge_step_mode: config.nudge_step_mode,
            nudge_step_px: config.nudge_step_px,
            nudge_fast_step_px: config.nudge_fast_step_px,
            disable_keyboard_a11y: config.disable_keyboard_a11y,
            pane_click_distance: config.pane_click_distance,
            pan_activation_key_code: config.pan_activation_key_code,
            space_to_pan: config.space_to_pan,
            pan_on_scroll_speed: config.pan_on_scroll_speed,
            pan_on_scroll_mode: config.pan_on_scroll_mode,
            pan_inertia: config.pan_inertia.clone(),
            zoom_on_scroll: config.zoom_on_scroll,
            zoom_on_scroll_speed: config.zoom_on_scroll_speed,
            zoom_on_pinch: config.zoom_on_pinch,
            zoom_on_pinch_speed: config.zoom_on_pinch_speed,
            zoom_on_double_click: config.zoom_on_double_click,
            frame_view_duration_ms: config.frame_view_duration_ms,
            frame_view_interpolate: config.frame_view_interpolate,
            frame_view_ease: config.frame_view_ease,
            frame_view_padding: config.frame_view_padding,
            reroute_on_edge_double_click: config.reroute_on_edge_double_click,
            edge_insert_on_alt_drag: config.edge_insert_on_alt_drag,
            zoom_activation_key: config.zoom_activation_key,
            node_drag_threshold: config.node_drag_threshold,
            node_drag_handle_mode: config.node_drag_handle_mode,
            node_click_distance: config.node_click_distance,
            connection_drag_threshold: config.connection_drag_threshold,
            connect_on_click: config.connect_on_click,
            auto_pan: config.auto_pan.clone(),
            translate_extent: config.translate_extent,
            node_extent: config.node_extent,
            node_origin: config.node_origin,
        }
    }

    pub fn config(&self) -> NodeGraphInteractionConfig {
        NodeGraphInteractionConfig {
            elements_selectable: self.elements_selectable,
            nodes_draggable: self.nodes_draggable,
            nodes_connectable: self.nodes_connectable,
            nodes_deletable: self.nodes_deletable,
            edges_selectable: self.edges_selectable,
            edges_deletable: self.edges_deletable,
            edges_focusable: self.edges_focusable,
            edges_reconnectable: self.edges_reconnectable,
            connection_mode: self.connection_mode,
            connection_radius: self.connection_radius,
            reconnect_radius: self.reconnect_radius,
            reconnect_on_drop_empty: self.reconnect_on_drop_empty,
            edge_interaction_width: self.edge_interaction_width,
            bezier_hit_test_steps: self.bezier_hit_test_steps,
            elevate_nodes_on_select: self.elevate_nodes_on_select,
            elevate_edges_on_select: self.elevate_edges_on_select,
            snap_to_grid: self.snap_to_grid,
            snap_grid: self.snap_grid,
            snaplines: self.snaplines,
            snaplines_threshold: self.snaplines_threshold,
            pan_on_scroll: self.pan_on_scroll,
            pan_on_drag: self.pan_on_drag,
            selection_on_drag: self.selection_on_drag,
            selection_mode: self.selection_mode,
            box_select_edges: self.box_select_edges,
            selection_key: self.selection_key,
            multi_selection_key: self.multi_selection_key,
            delete_key: self.delete_key,
            nudge_step_mode: self.nudge_step_mode,
            nudge_step_px: self.nudge_step_px,
            nudge_fast_step_px: self.nudge_fast_step_px,
            disable_keyboard_a11y: self.disable_keyboard_a11y,
            pane_click_distance: self.pane_click_distance,
            pan_activation_key_code: self.pan_activation_key_code,
            space_to_pan: self.space_to_pan,
            pan_on_scroll_speed: self.pan_on_scroll_speed,
            pan_on_scroll_mode: self.pan_on_scroll_mode,
            pan_inertia: self.pan_inertia.clone(),
            zoom_on_scroll: self.zoom_on_scroll,
            zoom_on_scroll_speed: self.zoom_on_scroll_speed,
            zoom_on_pinch: self.zoom_on_pinch,
            zoom_on_pinch_speed: self.zoom_on_pinch_speed,
            zoom_on_double_click: self.zoom_on_double_click,
            frame_view_duration_ms: self.frame_view_duration_ms,
            frame_view_interpolate: self.frame_view_interpolate,
            frame_view_ease: self.frame_view_ease,
            frame_view_padding: self.frame_view_padding,
            reroute_on_edge_double_click: self.reroute_on_edge_double_click,
            edge_insert_on_alt_drag: self.edge_insert_on_alt_drag,
            zoom_activation_key: self.zoom_activation_key,
            node_drag_threshold: self.node_drag_threshold,
            node_drag_handle_mode: self.node_drag_handle_mode,
            node_click_distance: self.node_click_distance,
            connection_drag_threshold: self.connection_drag_threshold,
            connect_on_click: self.connect_on_click,
            auto_pan: self.auto_pan.clone(),
            translate_extent: self.translate_extent,
            node_extent: self.node_extent,
            node_origin: self.node_origin,
        }
    }

    pub fn runtime_tuning(&self) -> NodeGraphRuntimeTuning {
        NodeGraphRuntimeTuning {
            spatial_index: self.spatial_index,
            only_render_visible_elements: self.only_render_visible_elements,
            paint_cache_prune: self.paint_cache_prune,
        }
    }

    pub fn split(&self) -> (NodeGraphInteractionConfig, NodeGraphRuntimeTuning) {
        (self.config(), self.runtime_tuning())
    }
}

impl Default for NodeGraphInteractionState {
    fn default() -> Self {
        Self {
            elements_selectable: default_elements_selectable(),
            nodes_draggable: default_nodes_draggable(),
            nodes_connectable: default_nodes_connectable(),
            nodes_deletable: default_nodes_deletable(),
            edges_selectable: default_edges_selectable(),
            edges_deletable: default_edges_deletable(),
            edges_focusable: default_edges_focusable(),
            edges_reconnectable: default_edges_reconnectable(),
            connection_mode: NodeGraphConnectionMode::default(),
            connection_radius: default_connection_radius(),
            reconnect_radius: default_reconnect_radius(),
            reconnect_on_drop_empty: false,
            edge_interaction_width: default_edge_interaction_width(),
            bezier_hit_test_steps: default_bezier_hit_test_steps(),
            spatial_index: NodeGraphSpatialIndexTuning::default(),
            only_render_visible_elements: default_only_render_visible_elements(),
            elevate_nodes_on_select: default_elevate_nodes_on_select(),
            elevate_edges_on_select: default_elevate_edges_on_select(),
            paint_cache_prune: NodeGraphPaintCachePruneTuning::default(),
            snap_to_grid: false,
            snap_grid: default_snap_grid(),
            snaplines: default_snaplines(),
            snaplines_threshold: default_snaplines_threshold(),
            pan_on_scroll: default_pan_on_scroll(),
            pan_on_drag: default_pan_on_drag_buttons(),
            selection_on_drag: false,
            selection_mode: NodeGraphSelectionMode::default(),
            box_select_edges: default_box_select_edges(),
            selection_key: default_selection_key(),
            multi_selection_key: default_multi_selection_key(),
            delete_key: NodeGraphDeleteKey::default(),
            nudge_step_mode: NodeGraphNudgeStepMode::default(),
            nudge_step_px: default_nudge_step_px(),
            nudge_fast_step_px: default_nudge_fast_step_px(),
            disable_keyboard_a11y: false,
            pane_click_distance: default_pane_click_distance(),
            pan_activation_key_code: default_pan_activation_key_code(),
            space_to_pan: default_space_to_pan(),
            pan_on_scroll_speed: default_pan_on_scroll_speed(),
            pan_on_scroll_mode: NodeGraphPanOnScrollMode::default(),
            pan_inertia: NodeGraphPanInertiaTuning::default(),
            zoom_on_scroll: default_zoom_on_scroll(),
            zoom_on_scroll_speed: default_zoom_on_scroll_speed(),
            zoom_on_pinch: default_zoom_on_pinch(),
            zoom_on_pinch_speed: default_zoom_on_pinch_speed(),
            zoom_on_double_click: default_zoom_on_double_click(),
            frame_view_duration_ms: default_frame_view_duration_ms(),
            frame_view_interpolate: NodeGraphViewportInterpolate::default(),
            frame_view_ease: None,
            frame_view_padding: default_frame_view_padding(),
            reroute_on_edge_double_click: default_reroute_on_edge_double_click(),
            edge_insert_on_alt_drag: default_edge_insert_on_alt_drag(),
            zoom_activation_key: NodeGraphZoomActivationKey::default(),
            node_drag_threshold: default_node_drag_threshold(),
            node_drag_handle_mode: NodeGraphDragHandleMode::default(),
            node_click_distance: default_node_click_distance(),
            connection_drag_threshold: default_connection_drag_threshold(),
            connect_on_click: false,
            auto_pan: NodeGraphAutoPanTuning::default(),
            translate_extent: None,
            node_extent: None,
            node_origin: NodeGraphNodeOrigin::default(),
        }
    }
}

fn default_elevate_nodes_on_select() -> bool {
    false
}

fn default_elevate_edges_on_select() -> bool {
    true
}

fn default_nudge_step_px() -> f32 {
    1.0
}

fn default_nudge_fast_step_px() -> f32 {
    10.0
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphSpatialIndexTuning {
    /// Preferred cell size in screen pixels (converted to canvas units by dividing by zoom).
    #[serde(default = "NodeGraphSpatialIndexTuning::default_cell_size_screen_px")]
    pub cell_size_screen_px: f32,
    /// Minimum cell size in screen pixels (converted to canvas units by dividing by zoom).
    #[serde(default = "NodeGraphSpatialIndexTuning::default_min_cell_size_screen_px")]
    pub min_cell_size_screen_px: f32,
    /// Extra padding (screen px) applied to edge wire AABBs to ensure stable hit-test candidate sets.
    #[serde(default = "NodeGraphSpatialIndexTuning::default_edge_aabb_pad_screen_px")]
    pub edge_aabb_pad_screen_px: f32,
}

impl NodeGraphSpatialIndexTuning {
    fn default_cell_size_screen_px() -> f32 {
        256.0
    }

    fn default_min_cell_size_screen_px() -> f32 {
        16.0
    }

    fn default_edge_aabb_pad_screen_px() -> f32 {
        96.0
    }
}

impl Default for NodeGraphSpatialIndexTuning {
    fn default() -> Self {
        Self {
            cell_size_screen_px: Self::default_cell_size_screen_px(),
            min_cell_size_screen_px: Self::default_min_cell_size_screen_px(),
            edge_aabb_pad_screen_px: Self::default_edge_aabb_pad_screen_px(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphPaintCachePruneTuning {
    /// Remove cache entries not used within this many frames.
    #[serde(default = "NodeGraphPaintCachePruneTuning::default_max_age_frames")]
    pub max_age_frames: u64,
    /// Hard cap on total cache entries (paths + markers + text blobs + text metrics).
    #[serde(default = "NodeGraphPaintCachePruneTuning::default_max_entries")]
    pub max_entries: usize,
}

impl NodeGraphPaintCachePruneTuning {
    fn default_max_age_frames() -> u64 {
        300
    }

    fn default_max_entries() -> usize {
        30_000
    }
}

impl Default for NodeGraphPaintCachePruneTuning {
    fn default() -> Self {
        Self {
            max_age_frames: Self::default_max_age_frames(),
            max_entries: Self::default_max_entries(),
        }
    }
}

fn default_elements_selectable() -> bool {
    true
}

fn default_edges_selectable() -> bool {
    true
}

fn default_edges_focusable() -> bool {
    true
}

fn default_edges_reconnectable() -> bool {
    true
}

fn default_pan_on_scroll() -> bool {
    true
}

fn default_only_render_visible_elements() -> bool {
    true
}

fn default_space_to_pan() -> bool {
    true
}

fn default_selection_key() -> NodeGraphModifierKey {
    NodeGraphModifierKey::Shift
}

fn default_multi_selection_key() -> NodeGraphModifierKey {
    NodeGraphModifierKey::CtrlOrMeta
}

fn default_pan_activation_key_code() -> Option<NodeGraphKeyCode> {
    Some(NodeGraphKeyCode(fret_core::KeyCode::Space))
}

fn default_pane_click_distance() -> f32 {
    1.0
}

fn default_pan_on_scroll_speed() -> f32 {
    1.0
}

fn default_zoom_on_scroll() -> bool {
    true
}

fn default_zoom_on_scroll_speed() -> f32 {
    1.0
}

fn default_zoom_on_pinch() -> bool {
    true
}

fn default_zoom_on_pinch_speed() -> f32 {
    1.0
}

fn default_zoom_on_double_click() -> bool {
    true
}

fn default_frame_view_duration_ms() -> u32 {
    200
}

fn default_frame_view_padding() -> f32 {
    0.0
}

fn default_reroute_on_edge_double_click() -> bool {
    false
}

fn default_edge_insert_on_alt_drag() -> bool {
    false
}

fn default_connection_radius() -> f32 {
    16.0
}

fn default_reconnect_radius() -> f32 {
    10.0
}

fn default_edge_interaction_width() -> f32 {
    12.0
}

fn default_snap_grid() -> CanvasSize {
    CanvasSize {
        width: 16.0,
        height: 16.0,
    }
}

fn default_snaplines() -> bool {
    true
}

fn default_snaplines_threshold() -> f32 {
    8.0
}

fn default_node_drag_threshold() -> f32 {
    1.0
}

fn default_node_click_distance() -> f32 {
    2.0
}

fn default_connection_drag_threshold() -> f32 {
    2.0
}

/// Errors for reading/writing view-state files.
#[derive(Debug, thiserror::Error)]
pub enum NodeGraphViewStateFileError {
    /// Read failure.
    #[error("failed to read node graph view-state file: {path}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    /// JSON parse failure.
    #[error("failed to parse node graph view-state file JSON: {path}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    /// Write failure.
    #[error("failed to write node graph view-state file: {path}")]
    Write {
        path: String,
        source: std::io::Error,
    },
    /// JSON serialization failure.
    #[error("failed to serialize node graph view-state JSON: {path}")]
    Serialize {
        path: String,
        source: serde_json::Error,
    },
    /// Wrapper id mismatch.
    #[error("view-state file wrapper graph_id does not match requested graph_id")]
    InconsistentGraphId,
}

/// View-state persistence file helper.
///
/// The type name is historical, but the emitted on-disk wrapper now follows `state_version = 2` and
/// stores pure view-state separately from persisted interaction config and runtime tuning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphViewStateFileV1 {
    /// Graph id.
    pub graph_id: GraphId,
    /// View-state schema version.
    pub state_version: u32,
    /// Pure view-state payload.
    pub state: NodeGraphViewState,
    /// Persisted editor interaction configuration.
    #[serde(
        default,
        skip_serializing_if = "NodeGraphInteractionConfig::is_default"
    )]
    pub interaction: NodeGraphInteractionConfig,
    /// Persisted runtime tuning.
    #[serde(default, skip_serializing_if = "NodeGraphRuntimeTuning::is_default")]
    pub runtime_tuning: NodeGraphRuntimeTuning,
}

impl NodeGraphViewStateFileV1 {
    /// Wraps state for a graph.
    pub fn new(graph_id: GraphId, state: NodeGraphViewState) -> Self {
        Self::new_with_editor_config(graph_id, state, NodeGraphEditorConfig::default())
    }

    pub fn new_with_editor_config(
        graph_id: GraphId,
        state: NodeGraphViewState,
        editor_config: NodeGraphEditorConfig,
    ) -> Self {
        Self {
            graph_id,
            state_version: VIEW_STATE_VERSION,
            state,
            interaction: editor_config.interaction,
            runtime_tuning: editor_config.runtime_tuning,
        }
    }

    /// Loads a JSON file.
    ///
    /// Backward compatibility: accepts both the wrapped form and a plain `NodeGraphViewState` root
    /// object when the `graph_id` is supplied out-of-band (ADR 0126).
    pub fn load_json(
        path: impl AsRef<Path>,
        graph_id: GraphId,
    ) -> Result<Self, NodeGraphViewStateFileError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| NodeGraphViewStateFileError::Read {
            path: path.display().to_string(),
            source,
        })?;

        let root: serde_json::Value = serde_json::from_slice(&bytes).map_err(|source| {
            NodeGraphViewStateFileError::Parse {
                path: path.display().to_string(),
                source,
            }
        })?;

        if root.get("graph_id").is_some() && root.get("state").is_some() {
            #[derive(Deserialize)]
            struct WrappedViewStateFile {
                graph_id: GraphId,
                state_version: u32,
                state: serde_json::Value,
                #[serde(default)]
                interaction: Option<serde_json::Value>,
                #[serde(default)]
                runtime_tuning: Option<serde_json::Value>,
            }

            let wrapped: WrappedViewStateFile = serde_json::from_value(root).map_err(|source| {
                NodeGraphViewStateFileError::Parse {
                    path: path.display().to_string(),
                    source,
                }
            })?;
            if wrapped.graph_id != graph_id {
                return Err(NodeGraphViewStateFileError::InconsistentGraphId);
            }
            let loaded = if wrapped.state_version >= 2
                || wrapped.interaction.is_some()
                || wrapped.runtime_tuning.is_some()
            {
                parse_wrapped_view_state_json_values(
                    wrapped.state,
                    wrapped.interaction,
                    wrapped.runtime_tuning,
                )
            } else {
                parse_view_state_json_value(wrapped.state)
            }
            .map_err(|source| NodeGraphViewStateFileError::Parse {
                path: path.display().to_string(),
                source,
            })?;
            return Ok(Self {
                graph_id: wrapped.graph_id,
                state_version: wrapped.state_version,
                state: loaded.state,
                interaction: loaded.interaction,
                runtime_tuning: loaded.runtime_tuning,
            });
        }

        let loaded = parse_view_state_json_value(root).map_err(|source| {
            NodeGraphViewStateFileError::Parse {
                path: path.display().to_string(),
                source,
            }
        })?;
        Ok(Self {
            graph_id,
            state_version: VIEW_STATE_VERSION,
            state: loaded.state,
            interaction: loaded.interaction,
            runtime_tuning: loaded.runtime_tuning,
        })
    }

    /// Loads the JSON file if it exists.
    pub fn load_json_if_exists(
        path: impl AsRef<Path>,
        graph_id: GraphId,
    ) -> Result<Option<Self>, NodeGraphViewStateFileError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(None);
        }
        Self::load_json(path, graph_id).map(Some)
    }

    /// Saves the JSON file (pretty-printed).
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), NodeGraphViewStateFileError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                NodeGraphViewStateFileError::Write {
                    path: path.display().to_string(),
                    source,
                }
            })?;
        }
        let persisted = PersistedNodeGraphViewStateFileV2 {
            graph_id: self.graph_id,
            state_version: VIEW_STATE_VERSION,
            state: NodeGraphPureViewState::from(&self.state),
            interaction: self.interaction.clone(),
            runtime_tuning: self.runtime_tuning,
        };
        let bytes = serde_json::to_vec_pretty(&persisted).map_err(|source| {
            NodeGraphViewStateFileError::Serialize {
                path: path.display().to_string(),
                source,
            }
        })?;
        std::fs::write(path, bytes).map_err(|source| NodeGraphViewStateFileError::Write {
            path: path.display().to_string(),
            source,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct PersistedNodeGraphViewStateFileV2 {
    graph_id: GraphId,
    state_version: u32,
    state: NodeGraphPureViewState,
    #[serde(
        default,
        skip_serializing_if = "NodeGraphInteractionConfig::is_default"
    )]
    interaction: NodeGraphInteractionConfig,
    #[serde(default, skip_serializing_if = "NodeGraphRuntimeTuning::is_default")]
    runtime_tuning: NodeGraphRuntimeTuning,
}

struct ParsedNodeGraphViewDocument {
    state: NodeGraphViewState,
    interaction: NodeGraphInteractionConfig,
    runtime_tuning: NodeGraphRuntimeTuning,
}

fn parse_wrapped_view_state_json_values(
    state: serde_json::Value,
    interaction: Option<serde_json::Value>,
    runtime_tuning: Option<serde_json::Value>,
) -> Result<ParsedNodeGraphViewDocument, serde_json::Error> {
    let state: NodeGraphPureViewState = serde_json::from_value(state)?;
    let (interaction, migrated_runtime_tuning) = parse_interaction_config_json_value(interaction)?;
    let runtime_tuning = if let Some(value) = runtime_tuning {
        serde_json::from_value(value)?
    } else {
        migrated_runtime_tuning
    };
    let state = NodeGraphViewState::from(state);
    Ok(ParsedNodeGraphViewDocument {
        state,
        interaction,
        runtime_tuning,
    })
}

fn parse_interaction_config_json_value(
    value: Option<serde_json::Value>,
) -> Result<(NodeGraphInteractionConfig, NodeGraphRuntimeTuning), serde_json::Error> {
    let Some(value) = value else {
        return Ok((
            NodeGraphInteractionConfig::default(),
            NodeGraphRuntimeTuning::default(),
        ));
    };
    let looks_like_legacy_interaction_state = value.as_object().is_some_and(|map| {
        map.contains_key("spatial_index")
            || map.contains_key("only_render_visible_elements")
            || map.contains_key("paint_cache_prune")
    });
    if looks_like_legacy_interaction_state {
        return serde_json::from_value::<NodeGraphInteractionState>(value)
            .map(|legacy| legacy.split());
    }
    match serde_json::from_value::<NodeGraphInteractionConfig>(value.clone()) {
        Ok(config) => Ok((config, NodeGraphRuntimeTuning::default())),
        Err(config_err) => match serde_json::from_value::<NodeGraphInteractionState>(value) {
            Ok(legacy) => Ok(legacy.split()),
            Err(_) => Err(config_err),
        },
    }
}

fn parse_view_state_json_value(
    value: serde_json::Value,
) -> Result<ParsedNodeGraphViewDocument, serde_json::Error> {
    #[derive(Deserialize)]
    struct LegacyFlatViewState {
        #[serde(default)]
        pan: crate::core::CanvasPoint,
        #[serde(default = "default_zoom")]
        zoom: f32,
        #[serde(default)]
        selected_nodes: Vec<NodeId>,
        #[serde(default)]
        selected_edges: Vec<EdgeId>,
        #[serde(default)]
        selected_groups: Vec<GroupId>,
        #[serde(default)]
        draw_order: Vec<NodeId>,
        #[serde(default)]
        group_draw_order: Vec<GroupId>,
        #[serde(default)]
        interaction: Option<serde_json::Value>,
        #[serde(default)]
        runtime_tuning: Option<serde_json::Value>,
    }

    let legacy: LegacyFlatViewState = serde_json::from_value(value.clone())?;
    let (interaction, migrated_runtime_tuning) =
        parse_interaction_config_json_value(legacy.interaction)?;
    let runtime_tuning = if let Some(runtime_tuning) = legacy.runtime_tuning {
        serde_json::from_value(runtime_tuning)?
    } else {
        migrated_runtime_tuning
    };

    let state = NodeGraphViewState {
        pan: legacy.pan,
        zoom: legacy.zoom,
        selected_nodes: legacy.selected_nodes,
        selected_edges: legacy.selected_edges,
        selected_groups: legacy.selected_groups,
        draw_order: legacy.draw_order,
        group_draw_order: legacy.group_draw_order,
    };

    Ok(ParsedNodeGraphViewDocument {
        state,
        interaction,
        runtime_tuning,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str, graph_id: GraphId) -> PathBuf {
        std::env::temp_dir().join(format!("fret_node_{name}_{graph_id}.json"))
    }

    #[test]
    fn view_state_file_roundtrips() {
        let graph_id = GraphId::new();
        let path = temp_path("view_state_roundtrip", graph_id);

        let state = NodeGraphViewState {
            pan: crate::core::CanvasPoint { x: 12.5, y: -3.0 },
            zoom: 1.25,
            ..NodeGraphViewState::default()
        };
        let mut editor_config = NodeGraphEditorConfig::default();
        editor_config.interaction.selection_on_drag = true;
        editor_config.runtime_tuning.only_render_visible_elements = false;

        let file = NodeGraphViewStateFileV1::new_with_editor_config(
            graph_id,
            state.clone(),
            editor_config.clone(),
        );
        file.save_json(&path).unwrap();

        let root: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(root.get("state_version").and_then(|v| v.as_u64()), Some(2));
        assert!(root
            .get("state")
            .and_then(|v| v.get("interaction"))
            .is_none());
        assert!(root
            .get("state")
            .and_then(|v| v.get("runtime_tuning"))
            .is_none());
        assert_eq!(
            root.get("interaction")
                .and_then(|v| v.get("selection_on_drag"))
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            root.get("runtime_tuning")
                .and_then(|v| v.get("only_render_visible_elements"))
                .and_then(|v| v.as_bool()),
            Some(false)
        );

        let loaded = NodeGraphViewStateFileV1::load_json(&path, graph_id).unwrap();
        assert_eq!(loaded.graph_id, graph_id);
        assert_eq!(loaded.state.pan.x, state.pan.x);
        assert_eq!(loaded.state.pan.y, state.pan.y);
        assert_eq!(loaded.state.zoom, state.zoom);
        assert!(loaded.interaction.selection_on_drag);
        assert!(!loaded.runtime_tuning.only_render_visible_elements);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn view_state_file_accepts_plain_state_root() {
        let graph_id = GraphId::new();
        let path = temp_path("view_state_plain_root", graph_id);

        let state = NodeGraphViewState {
            pan: crate::core::CanvasPoint { x: 1.0, y: 2.0 },
            zoom: 0.75,
            ..NodeGraphViewState::default()
        };

        let bytes = serde_json::to_vec_pretty(&state).unwrap();
        std::fs::write(&path, bytes).unwrap();

        let loaded = NodeGraphViewStateFileV1::load_json(&path, graph_id).unwrap();
        assert_eq!(loaded.graph_id, graph_id);
        assert_eq!(loaded.state.pan.x, state.pan.x);
        assert_eq!(loaded.state.pan.y, state.pan.y);
        assert_eq!(loaded.state.zoom, state.zoom);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn interaction_state_split_roundtrips_runtime_tuning() {
        let mut interaction = NodeGraphInteractionState::default();
        interaction.selection_on_drag = true;
        interaction.only_render_visible_elements = false;
        interaction.spatial_index.edge_aabb_pad_screen_px = 123.0;
        interaction.paint_cache_prune.max_entries = 4_096;

        let (config, runtime_tuning) = interaction.split();
        assert!(config.selection_on_drag);
        assert!(!runtime_tuning.only_render_visible_elements);
        assert_eq!(runtime_tuning.spatial_index.edge_aabb_pad_screen_px, 123.0);
        assert_eq!(runtime_tuning.paint_cache_prune.max_entries, 4_096);

        let rebuilt = NodeGraphInteractionState::from_parts(&config, &runtime_tuning);
        assert_eq!(rebuilt, interaction);
    }

    #[test]
    fn view_state_file_migrates_legacy_runtime_tuning_from_interaction() {
        let graph_id = GraphId::new();
        let path = temp_path("view_state_legacy_runtime_tuning", graph_id);

        let mut legacy_interaction = NodeGraphInteractionState::default();
        legacy_interaction.selection_on_drag = true;
        legacy_interaction.only_render_visible_elements = false;
        legacy_interaction.spatial_index.edge_aabb_pad_screen_px = 222.0;
        legacy_interaction.paint_cache_prune.max_age_frames = 9;

        let state_json = serde_json::json!({
            "pan": { "x": 3.0, "y": 4.0 },
            "zoom": 1.5,
            "interaction": serde_json::to_value(&legacy_interaction).unwrap()
        });
        std::fs::write(&path, serde_json::to_vec_pretty(&state_json).unwrap()).unwrap();

        let loaded = NodeGraphViewStateFileV1::load_json(&path, graph_id).unwrap();
        assert_eq!(loaded.state.pan.x, 3.0);
        assert_eq!(loaded.state.pan.y, 4.0);
        assert_eq!(loaded.state.zoom, 1.5);
        assert!(loaded.interaction.selection_on_drag);
        assert!(!loaded.runtime_tuning.only_render_visible_elements);
        assert_eq!(
            loaded.runtime_tuning.spatial_index.edge_aabb_pad_screen_px,
            222.0
        );
        assert_eq!(loaded.runtime_tuning.paint_cache_prune.max_age_frames, 9);

        let resolved = NodeGraphEditorConfig {
            interaction: loaded.interaction.clone(),
            runtime_tuning: loaded.runtime_tuning,
        }
        .resolved_interaction_state();
        assert!(resolved.selection_on_drag);
        assert!(!resolved.only_render_visible_elements);
        assert_eq!(resolved.spatial_index.edge_aabb_pad_screen_px, 222.0);
        assert_eq!(resolved.paint_cache_prune.max_age_frames, 9);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn view_state_file_rejects_wrong_graph_id() {
        let graph_id = GraphId::new();
        let other = GraphId::new();
        let path = temp_path("view_state_wrong_graph_id", graph_id);

        let file = NodeGraphViewStateFileV1::new(graph_id, NodeGraphViewState::default());
        file.save_json(&path).unwrap();

        let err = NodeGraphViewStateFileV1::load_json(&path, other).unwrap_err();
        assert!(matches!(
            err,
            NodeGraphViewStateFileError::InconsistentGraphId
        ));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn view_state_sanitize_removes_stale_ids() {
        let graph_id = GraphId::new();
        let mut graph = Graph::new(graph_id);

        let keep_node = NodeId::new();
        graph.nodes.insert(
            keep_node,
            crate::core::Node {
                kind: crate::core::NodeKindKey::new("test"),
                kind_version: 1,
                pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        );

        let mut state = NodeGraphViewState {
            selected_nodes: vec![keep_node, NodeId::new()],
            draw_order: vec![NodeId::new(), keep_node],
            ..NodeGraphViewState::default()
        };

        state.sanitize_for_graph(&graph);
        assert_eq!(state.selected_nodes, vec![keep_node]);
        assert_eq!(state.draw_order, vec![keep_node]);
    }
}
