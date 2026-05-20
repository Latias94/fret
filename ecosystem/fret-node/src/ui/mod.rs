//! Fret UI integration for the node graph editor.
//!
//! This module is behind the `fret-ui` feature.
//!
//! Retained-canvas integration surfaces (the legacy widget/editor stack) are behind
//! `compat-retained-canvas` so downstream authors can adopt declarative UI without enabling
//! `fret-ui/unstable-retained-bridge`.

pub mod binding;
mod canvas;
pub mod commands;
mod compat_transport;
pub mod controller;
pub mod declarative;
pub mod edge_types;
pub mod geometry_overrides;
pub mod internals;
pub mod measured;
pub mod paint_overrides;
mod portal_commands;
pub mod portal_layout;
pub mod presenter;
pub mod presets;
pub mod registry;
pub mod skin;
pub mod style;
mod viewport_helper;
mod viewport_options;

#[cfg(feature = "compat-retained-canvas")]
mod a11y;
#[cfg(feature = "compat-retained-canvas")]
mod editor;
mod editors;
mod overlays;
#[cfg(feature = "compat-retained-canvas")]
mod panel;
#[cfg(feature = "compat-retained-canvas")]
mod portal;
#[cfg(feature = "compat-retained-canvas")]
mod retained_event_tail;
#[cfg(feature = "compat-retained-canvas")]
mod retained_submit;
mod screen_space_placement;

pub use binding::NodeGraphSurfaceBinding;
pub use canvas::NodeResizeHandle;
pub use commands::register_node_graph_commands;
pub use controller::{
    NodeGraphController, NodeGraphControllerError, NodeGraphEdgeUpdate,
    NodeGraphNodeConnectionsQuery, NodeGraphNodeUpdate, NodeGraphPortConnectionsQuery,
};
pub use declarative::{
    NodeGraphDiagnosticsConfig, NodeGraphSurfaceProps, NodeGraphVisibleSubsetPortalConfig,
    node_graph_surface, node_graph_surface_in,
};
pub use edge_types::{EdgeCustomPath, EdgePathInput, EdgeTypeKey, NodeGraphEdgeTypes};
pub use geometry_overrides::{
    EdgeGeometryOverrideV1, NodeGeometryOverrideV1, NodeGraphGeometryOverrides,
    NodeGraphGeometryOverridesMap, NodeGraphGeometryOverridesRef,
};
pub use internals::{
    NodeGraphCanvasTransform, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};
pub use measured::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter,
};
pub use paint_overrides::{
    EdgePaintOverrideV1, NodeGraphPaintOverrides, NodeGraphPaintOverridesMap,
    NodeGraphPaintOverridesRef, NodePaintOverrideV1,
};
pub use portal_layout::NodeGraphPortalNodeLayout;
pub use presenter::{
    DefaultNodeGraphPresenter, EdgeMarker, EdgeMarkerKind, EdgeRenderHint, EdgeRouteKind,
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter,
    NodeResizeConstraintsPx, NodeResizeHandleSet, PortAnchorHint, RegistryNodeGraphPresenter,
};
pub use presets::{NodeGraphPresetFamily, NodeGraphPresetSkinV1};
pub use registry::{NodeGraphNodeRenderer, NodeGraphNodeTypes};
pub use skin::{
    CanvasChromeHint, EdgeChromeHint, InteractionChromeHint, NodeChromeHint, NodeGraphSkin,
    NodeGraphSkinRef, NodeRingHint, NodeShadowHint, NoopNodeGraphSkin, PortChromeHint,
    PortShapeHint, WireGlowHint, WireHighlightHint, WireOutlineHint,
};
pub use style::{NodeGraphColorMode, NodeGraphStyle};
pub use viewport_options::{NodeGraphFitViewOptions, NodeGraphSetViewportOptions};

#[cfg(all(test, feature = "compat-retained-canvas"))]
pub(crate) use a11y::{
    NodeGraphA11yFocusedEdge, NodeGraphA11yFocusedNode, NodeGraphA11yFocusedPort,
};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub(crate) use canvas::{
    NodeGraphCanvas, NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome,
    NodeGraphCanvasMiddlewareCx,
};
#[cfg(all(test, feature = "compat-retained-canvas"))]
pub(crate) use editor::NodeGraphEditor;
