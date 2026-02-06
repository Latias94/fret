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
pub const VIEW_STATE_VERSION: u32 = 1;

/// Default project-scoped view-state path for a graph.
///
/// This follows ADR 0135's recommended `.fret/` layout.
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

/// Node graph editor view-state.
///
/// This is intentionally separate from graph semantics and may be stored per-user/per-project.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Optional interaction tuning (snap, connection mode, auto-pan, etc.).
    #[serde(default, skip_serializing_if = "NodeGraphInteractionState::is_default")]
    pub interaction: NodeGraphInteractionState,
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
            interaction: NodeGraphInteractionState::default(),
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

/// Optional interaction tuning persisted as part of editor view state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphInteractionState {
    /// Global master switch for selecting graph elements (nodes/edges/groups).
    #[serde(default = "default_elements_selectable")]
    pub elements_selectable: bool,

    /// Whether nodes can be dragged with pointer interactions (XyFlow `nodesDraggable`).
    #[serde(default = "default_nodes_draggable")]
    pub nodes_draggable: bool,

    /// Whether nodes can create/accept connections via editor interactions (XyFlow
    /// `nodesConnectable`).
    #[serde(default = "default_nodes_connectable")]
    pub nodes_connectable: bool,

    /// Whether nodes can be deleted via editor interactions (XyFlow `nodesDeletable`).
    #[serde(default = "default_nodes_deletable")]
    pub nodes_deletable: bool,

    /// Whether edges can be selected via pointer/keyboard.
    #[serde(default = "default_edges_selectable")]
    pub edges_selectable: bool,

    /// Whether edges can be deleted via editor interactions (XyFlow `edgesDeletable`).
    #[serde(default = "default_edges_deletable")]
    pub edges_deletable: bool,

    /// Whether edges can be focused via keyboard navigation.
    #[serde(default = "default_edges_focusable")]
    pub edges_focusable: bool,

    /// Whether edges can be reconnected by dragging edge update anchors.
    #[serde(default = "default_edges_reconnectable")]
    pub edges_reconnectable: bool,

    /// Connection targeting strategy.
    #[serde(default)]
    pub connection_mode: NodeGraphConnectionMode,

    /// Target search radius in screen pixels for loose connection mode.
    #[serde(default = "default_connection_radius")]
    pub connection_radius: f32,

    /// Reconnect anchor hit radius in screen pixels.
    #[serde(default = "default_reconnect_radius")]
    pub reconnect_radius: f32,

    /// Whether dropping an edge reconnect drag on empty canvas disconnects the edge.
    ///
    /// This matches common editor expectations (ShaderGraph/Blueprint) where dragging an edge end
    /// away from a port and releasing disconnects it.
    #[serde(default)]
    pub reconnect_on_drop_empty: bool,

    /// Edge hit slop width in screen pixels (independent from wire stroke thickness).
    #[serde(default = "default_edge_interaction_width")]
    pub edge_interaction_width: f32,

    /// Subdivision count used for Bezier wire hit-testing (higher is more accurate, but slower).
    #[serde(default = "default_bezier_hit_test_steps")]
    pub bezier_hit_test_steps: u8,

    /// Spatial index tuning for hit-testing and culling.
    #[serde(default = "default_spatial_index_tuning")]
    pub spatial_index: NodeGraphSpatialIndexTuning,

    /// Whether to only process/render elements near the visible viewport (XyFlow
    /// `onlyRenderVisibleElements`).
    ///
    /// Note: Fret's canvas is always clipped to the viewport, but this flag controls whether we
    /// cull work (geometry queries, render data collection) to a padded viewport rect.
    #[serde(default = "default_only_render_visible_elements")]
    pub only_render_visible_elements: bool,

    /// Whether selected nodes should be drawn on top of non-selected nodes (XyFlow
    /// `elevateNodesOnSelect`).
    #[serde(default = "default_elevate_nodes_on_select")]
    pub elevate_nodes_on_select: bool,

    /// Whether selected edges should be drawn on top of non-selected edges (XyFlow
    /// `elevateEdgesOnSelect`).
    ///
    /// Note: when disabled, the canvas may choose to forego static edge caches so that selection
    /// styling can still be applied without changing the edge z-order.
    #[serde(default = "default_elevate_edges_on_select")]
    pub elevate_edges_on_select: bool,

    /// Paint-cache pruning tuning for long-lived graphs.
    #[serde(default = "default_paint_cache_prune_tuning")]
    pub paint_cache_prune: NodeGraphPaintCachePruneTuning,

    /// Snap nodes to a grid during move/resize interactions.
    #[serde(default)]
    pub snap_to_grid: bool,

    /// Snap grid size in canvas units.
    #[serde(default = "default_snap_grid")]
    pub snap_grid: CanvasSize,

    /// Show alignment guides and snap node moves to them.
    #[serde(default = "default_snaplines")]
    pub snaplines: bool,

    /// Snaplines threshold in screen pixels.
    #[serde(default = "default_snaplines_threshold")]
    pub snaplines_threshold: f32,

    /// Enables panning the canvas via wheel / touchpad scroll (XyFlow `panOnScroll`).
    #[serde(default = "default_pan_on_scroll")]
    pub pan_on_scroll: bool,

    /// Configures which mouse buttons may pan the canvas via drag (XyFlow `panOnDrag`).
    #[serde(default = "default_pan_on_drag_buttons")]
    pub pan_on_drag: NodeGraphPanOnDragButtons,

    /// Select multiple elements with a selection box without holding down Shift.
    ///
    /// This matches XyFlow's `selectionOnDrag`.
    #[serde(default)]
    pub selection_on_drag: bool,

    /// Selection behavior for marquee selection (XyFlow `selectionMode`).
    #[serde(default)]
    pub selection_mode: NodeGraphSelectionMode,

    /// How to select edges when marquee-selecting nodes (XyFlow behavior).
    ///
    /// Backward compatibility: this field accepts either:
    /// - a bool (`true` => `connected`, `false` => `none`), or
    /// - a snake_case string (`none`, `connected`, `both_endpoints`).
    #[serde(
        default = "default_box_select_edges",
        alias = "box_select_connected_edges"
    )]
    pub box_select_edges: NodeGraphBoxSelectEdges,

    /// Modifier used to activate selection box interactions (XyFlow `selectionKeyCode`).
    ///
    /// Default: Shift.
    #[serde(default = "default_selection_key")]
    pub selection_key: NodeGraphModifierKey,

    /// Modifier used to toggle/add to the current selection (XyFlow `multiSelectionKeyCode`).
    ///
    /// Default: Ctrl/Cmd.
    #[serde(default = "default_multi_selection_key")]
    pub multi_selection_key: NodeGraphModifierKey,

    /// Key used to delete the current selection (XyFlow `deleteKeyCode`).
    ///
    /// Default: Backspace.
    #[serde(default)]
    pub delete_key: NodeGraphDeleteKey,

    /// Nudge step mode for keyboard arrow movement (screen px vs snap grid step).
    #[serde(default)]
    pub nudge_step_mode: NodeGraphNudgeStepMode,

    /// Base nudge step in screen pixels when `nudge_step_mode` is `screen_px`.
    #[serde(default = "default_nudge_step_px")]
    pub nudge_step_px: f32,

    /// Fast nudge step in screen pixels when `nudge_step_mode` is `screen_px`.
    #[serde(default = "default_nudge_fast_step_px")]
    pub nudge_fast_step_px: f32,

    /// Disable keyboard-driven accessibility and focus traversal (XyFlow `disableKeyboardA11y`).
    ///
    /// When enabled, the canvas will avoid handling keyboard a11y navigation such as:
    /// - Tab-based focus traversal (nodes / edges),
    /// - Arrow-key nudging for selected nodes.
    ///
    /// Editor shortcuts such as delete/copy/paste are still handled.
    /// Overlay UIs (searcher, context menus) still receive keyboard events, and Escape can still
    /// cancel in-progress interactions.
    #[serde(default)]
    pub disable_keyboard_a11y: bool,

    /// Background click distance threshold in screen pixels (XyFlow `paneClickDistance`).
    ///
    /// This controls when a background drag transitions from a "click" into an interaction
    /// (marquee selection / panning / context menu suppression).
    #[serde(default = "default_pane_click_distance")]
    pub pane_click_distance: f32,

    /// Optional key code that activates panning while held down (XyFlow `panActivationKeyCode`).
    ///
    /// This is gated by `space_to_pan` for backward compatibility.
    ///
    /// Default: `Some("Space")`.
    #[serde(
        default = "default_pan_activation_key_code",
        skip_serializing_if = "Option::is_none"
    )]
    pub pan_activation_key_code: Option<NodeGraphKeyCode>,

    /// Enables panning the canvas by holding an activation key and dragging with the left mouse.
    ///
    /// This matches XyFlow's default "space-to-pan" editor affordance. The actual key is
    /// configured by `pan_activation_key_code` (default: Space). The name is kept for backward
    /// compatibility with early fret-node APIs.
    #[serde(default = "default_space_to_pan")]
    pub space_to_pan: bool,

    /// Wheel panning speed multiplier.
    #[serde(default = "default_pan_on_scroll_speed")]
    pub pan_on_scroll_speed: f32,

    /// Limits the direction of panning when `pan_on_scroll` is enabled (XyFlow `panOnScrollMode`).
    #[serde(default)]
    pub pan_on_scroll_mode: NodeGraphPanOnScrollMode,

    /// Optional inertial panning when finishing a pan gesture.
    #[serde(default)]
    pub pan_inertia: NodeGraphPanInertiaTuning,

    /// Whether wheel zoom is enabled at all.
    #[serde(default = "default_zoom_on_scroll")]
    pub zoom_on_scroll: bool,

    /// Wheel zoom speed multiplier.
    #[serde(default = "default_zoom_on_scroll_speed")]
    pub zoom_on_scroll_speed: f32,

    /// Whether pinch gesture zoom is enabled (XyFlow `zoomOnPinch`).
    #[serde(default = "default_zoom_on_pinch")]
    pub zoom_on_pinch: bool,

    /// Pinch gesture zoom speed multiplier.
    #[serde(default = "default_zoom_on_pinch_speed")]
    pub zoom_on_pinch_speed: f32,

    /// Whether double-click zoom is enabled (XyFlow `zoomOnDoubleClick`).
    #[serde(default = "default_zoom_on_double_click")]
    pub zoom_on_double_click: bool,

    /// Duration (ms) for view framing / fit-view style commands.
    ///
    /// Lightweight parity knob for XyFlow's `fitViewOptions.duration`.
    #[serde(default = "default_frame_view_duration_ms")]
    pub frame_view_duration_ms: u32,

    /// Interpolation style for view framing / fit-view style commands.
    ///
    /// Lightweight parity knob for XyFlow's `fitViewOptions.interpolate`.
    #[serde(default)]
    pub frame_view_interpolate: NodeGraphViewportInterpolate,

    /// Optional easing curve for view framing / fit-view style commands.
    ///
    /// Parity knob for XyFlow's `fitViewOptions.ease`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_view_ease: Option<NodeGraphViewportEase>,

    /// Optional extra padding when framing nodes (XyFlow `fitViewOptions.padding`).
    ///
    /// This is a fraction of the current viewport size (0.0 .. 0.45 recommended). When set to
    /// `0.0`, the framing logic falls back to its legacy fixed pixel margin.
    #[serde(default = "default_frame_view_padding")]
    pub frame_view_padding: f32,

    /// Whether double-clicking a wire inserts a reroute node (ShaderGraph-style).
    ///
    /// This is separate from `zoom_on_double_click`: zoom only applies when the double click hits
    /// the background, while reroute insertion applies when the double click hits an edge.
    #[serde(default = "default_reroute_on_edge_double_click")]
    pub reroute_on_edge_double_click: bool,

    /// Whether Alt-dragging a wire opens the insert-node picker on release (ShaderGraph-style).
    ///
    /// This is intentionally disabled by default to preserve XyFlow-like semantics.
    #[serde(default = "default_edge_insert_on_alt_drag")]
    pub edge_insert_on_alt_drag: bool,

    /// Modifier requirement for wheel zoom (XyFlow `zoomActivationKey`).
    #[serde(default)]
    pub zoom_activation_key: NodeGraphZoomActivationKey,

    /// Drag threshold in screen pixels before a node drag starts.
    #[serde(default = "default_node_drag_threshold")]
    pub node_drag_threshold: f32,

    /// Where node dragging can start (XyFlow `node.dragHandle` mental model).
    #[serde(default)]
    pub node_drag_handle_mode: NodeGraphDragHandleMode,

    /// Click tolerance in screen pixels for node selection gestures.
    ///
    /// When a pointer-down on a node does not exceed this distance before pointer-up, the action
    /// is treated as a click (useful for modifier-based selection toggles).
    ///
    /// This is similar to XyFlow's `nodeClickDistance` (d3-drag `clickDistance`).
    #[serde(default = "default_node_click_distance")]
    pub node_click_distance: f32,

    /// Drag threshold in screen pixels before a connection drag starts.
    #[serde(default = "default_connection_drag_threshold")]
    pub connection_drag_threshold: f32,

    /// Enables click-to-connect behavior (XyFlow `connectOnClick`).
    ///
    /// When enabled, a click on a port handle starts a connection preview; the next handle click
    /// attempts to create a connection and ends the click-connect session (regardless of validity).
    #[serde(default)]
    pub connect_on_click: bool,

    /// Auto-pan configuration.
    #[serde(default)]
    pub auto_pan: NodeGraphAutoPanTuning,

    /// Optional bounds for panning the viewport (XyFlow `translateExtent`).
    ///
    /// This is expressed in canvas coordinates and constrains the visible viewport rectangle
    /// (in canvas space) to stay within this extent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translate_extent: Option<CanvasRect>,

    /// Optional bounds for moving/resizing nodes (XyFlow `nodeExtent`).
    ///
    /// This is expressed in canvas coordinates and constrains node rectangles to stay within the
    /// extent. Parent groups may further constrain movement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_extent: Option<CanvasRect>,

    /// Node origin (anchor) used to interpret `Node.pos` (XyFlow `nodeOrigin`).
    #[serde(default)]
    pub node_origin: NodeGraphNodeOrigin,
}

impl NodeGraphInteractionState {
    fn is_default(this: &Self) -> bool {
        this == &Self::default()
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

/// View-state persistence file (v1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGraphViewStateFileV1 {
    /// Graph id.
    pub graph_id: GraphId,
    /// View-state schema version.
    pub state_version: u32,
    /// View-state payload.
    pub state: NodeGraphViewState,
}

impl NodeGraphViewStateFileV1 {
    /// Wraps state for a graph.
    pub fn new(graph_id: GraphId, state: NodeGraphViewState) -> Self {
        Self {
            graph_id,
            state_version: VIEW_STATE_VERSION,
            state,
        }
    }

    /// Loads a JSON file.
    ///
    /// Backward compatibility: accepts both the wrapped form and a plain `NodeGraphViewState` root
    /// object when the `graph_id` is supplied out-of-band (ADR 0135).
    pub fn load_json(
        path: impl AsRef<Path>,
        graph_id: GraphId,
    ) -> Result<Self, NodeGraphViewStateFileError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| NodeGraphViewStateFileError::Read {
            path: path.display().to_string(),
            source,
        })?;

        match serde_json::from_slice::<Self>(&bytes) {
            Ok(v) => {
                if v.graph_id != graph_id {
                    return Err(NodeGraphViewStateFileError::InconsistentGraphId);
                }
                Ok(v)
            }
            Err(new_err) => match serde_json::from_slice::<NodeGraphViewState>(&bytes) {
                Ok(state) => Ok(Self::new(graph_id, state)),
                Err(_old_err) => Err(NodeGraphViewStateFileError::Parse {
                    path: path.display().to_string(),
                    source: new_err,
                }),
            },
        }
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
        let bytes = serde_json::to_vec_pretty(self).map_err(|source| {
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

        let file = NodeGraphViewStateFileV1::new(graph_id, state.clone());
        file.save_json(&path).unwrap();

        let loaded = NodeGraphViewStateFileV1::load_json(&path, graph_id).unwrap();
        assert_eq!(loaded.graph_id, graph_id);
        assert_eq!(loaded.state.pan.x, state.pan.x);
        assert_eq!(loaded.state.pan.y, state.pan.y);
        assert_eq!(loaded.state.zoom, state.zoom);

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
