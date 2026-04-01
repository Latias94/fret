//! Fret UI integration for the node graph editor.
//!
//! This module is behind the `fret-ui` feature.
//!
//! Retained-canvas integration surfaces (the legacy widget/editor stack) are behind
//! `compat-retained-canvas` so downstream authors can adopt declarative UI without enabling
//! `fret-ui/unstable-retained-bridge`.

pub mod binding;
pub mod canvas;
pub mod commands;
mod compat_transport;
pub mod controller;
pub mod declarative;
pub mod edge_types;
pub mod geometry_overrides;
pub mod internals;
pub mod measured;
pub mod paint_overrides;
pub mod portal_layout;
pub mod presenter;
pub mod presets;
pub mod registry;
pub mod skin;
pub mod style;
mod viewport_helper;
mod viewport_options;

#[cfg(feature = "compat-retained-canvas")]
pub mod a11y;
#[cfg(feature = "compat-retained-canvas")]
pub mod diag_anchors;
#[cfg(feature = "compat-retained-canvas")]
pub mod editor;
#[cfg(feature = "compat-retained-canvas")]
pub mod editors;
#[cfg(feature = "compat-retained-canvas")]
pub mod overlays;
#[cfg(feature = "compat-retained-canvas")]
pub mod panel;
#[cfg(feature = "compat-retained-canvas")]
pub mod portal;
#[cfg(feature = "compat-retained-canvas")]
mod screen_space_placement;

pub use binding::NodeGraphSurfaceBinding;
#[cfg(feature = "compat-retained-canvas")]
pub use canvas::NodeGraphCanvas;
#[cfg(feature = "compat-retained-canvas")]
pub use canvas::NodeGraphCanvasWith;
pub use canvas::NodeResizeHandle;
#[cfg(feature = "compat-retained-canvas")]
pub use canvas::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareChain, NodeGraphCanvasMiddlewareCx,
    NoopNodeGraphCanvasMiddleware,
};
pub use commands::register_node_graph_commands;
pub use controller::{
    NodeGraphController, NodeGraphControllerError, NodeGraphEdgeUpdate,
    NodeGraphNodeConnectionsQuery, NodeGraphNodeUpdate, NodeGraphPortConnectionsQuery,
};
pub use declarative::{NodeGraphSurfaceProps, node_graph_surface};
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

#[cfg(feature = "compat-retained-canvas")]
pub use a11y::NodeGraphA11yActiveDescendant;
#[cfg(feature = "compat-retained-canvas")]
pub use a11y::{NodeGraphA11yFocusedEdge, NodeGraphA11yFocusedNode, NodeGraphA11yFocusedPort};
#[cfg(feature = "compat-retained-canvas")]
pub use declarative::{NodeGraphSurfaceCompatRetainedProps, node_graph_surface_compat_retained};
#[cfg(feature = "compat-retained-canvas")]
pub use diag_anchors::{NodeGraphDiagAnchor, NodeGraphDiagConnectingFlag};
#[cfg(feature = "compat-retained-canvas")]
pub use editor::NodeGraphEditor;
#[cfg(feature = "compat-retained-canvas")]
pub use editors::{
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
    PortalTextEditorUi,
};
#[cfg(feature = "compat-retained-canvas")]
pub use overlays::{
    GroupRenameOverlay, NodeGraphBlackboardOverlay, NodeGraphControlsBindings,
    NodeGraphControlsCommandBinding, NodeGraphControlsOverlay, NodeGraphEdgeToolbar,
    NodeGraphMiniMapBindings, NodeGraphMiniMapNavigationBinding, NodeGraphMiniMapOverlay,
    NodeGraphNodeToolbar, NodeGraphOverlayHost, NodeGraphOverlayState, NodeGraphToolbarAlign,
    NodeGraphToolbarPosition, NodeGraphToolbarSize, NodeGraphToolbarVisibility,
    SymbolRenameOverlay,
};
#[cfg(feature = "compat-retained-canvas")]
pub use panel::{NodeGraphPanel, NodeGraphPanelPosition, NodeGraphPanelSize};
#[cfg(feature = "compat-retained-canvas")]
pub use portal::{
    CMD_CANCEL_TEXT_PREFIX, CMD_STEP_TEXT_PREFIX, CMD_SUBMIT_TEXT_PREFIX,
    NodeGraphPortalCommandHandler, NodeGraphPortalHost, PortalCommandHandlerChain,
    PortalCommandOutcome, PortalNoopCommandHandler, PortalTextCommand, PortalTextStepMode,
    parse_portal_text_command, portal_cancel_text_command, portal_step_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
