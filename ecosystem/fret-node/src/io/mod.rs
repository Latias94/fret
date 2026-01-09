//! On-disk wrapper formats and optional helpers.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::{CanvasRect, CanvasSize, EdgeId, Graph, GraphId, GroupId, NodeId};

/// Graph file format version (v1).
pub const GRAPH_FILE_VERSION: u32 = 1;

/// Editor view-state format version (v1).
pub const VIEW_STATE_VERSION: u32 = 1;

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

fn default_zoom() -> f32 {
    1.0
}

/// Connection mode for selecting target ports during connection gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeGraphConnectionMode {
    Strict,
    Loose,
}

impl Default for NodeGraphConnectionMode {
    fn default() -> Self {
        Self::Strict
    }
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

/// Optional interaction tuning persisted as part of editor view state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeGraphInteractionState {
    /// Connection targeting strategy.
    #[serde(default)]
    pub connection_mode: NodeGraphConnectionMode,

    /// Target search radius in screen pixels for loose connection mode.
    #[serde(default = "default_connection_radius")]
    pub connection_radius: f32,

    /// Reconnect anchor hit radius in screen pixels.
    #[serde(default = "default_reconnect_radius")]
    pub reconnect_radius: f32,

    /// Edge hit slop width in screen pixels (independent from wire stroke thickness).
    #[serde(default = "default_edge_interaction_width")]
    pub edge_interaction_width: f32,

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

    /// Drag threshold in screen pixels before a node drag starts.
    #[serde(default = "default_node_drag_threshold")]
    pub node_drag_threshold: f32,

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
}

impl NodeGraphInteractionState {
    fn is_default(this: &Self) -> bool {
        this == &Self::default()
    }
}

impl Default for NodeGraphInteractionState {
    fn default() -> Self {
        Self {
            connection_mode: NodeGraphConnectionMode::default(),
            connection_radius: default_connection_radius(),
            reconnect_radius: default_reconnect_radius(),
            edge_interaction_width: default_edge_interaction_width(),
            snap_to_grid: false,
            snap_grid: default_snap_grid(),
            snaplines: default_snaplines(),
            snaplines_threshold: default_snaplines_threshold(),
            node_drag_threshold: default_node_drag_threshold(),
            node_click_distance: default_node_click_distance(),
            connection_drag_threshold: default_connection_drag_threshold(),
            connect_on_click: false,
            auto_pan: NodeGraphAutoPanTuning::default(),
            translate_extent: None,
            node_extent: None,
        }
    }
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
}
