//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.

pub mod a11y;
pub mod canvas;
pub mod commands;
pub mod edge_types;
pub mod edit_queue;
pub mod editor;
pub mod editors;
pub mod internals;
pub mod measured;
pub mod overlays;
pub mod panel;
pub mod portal;
pub mod presenter;
pub mod registry;
pub mod style;
pub mod view_queue;
pub mod viewport_helper;

pub use a11y::NodeGraphA11yActiveDescendant;
pub use a11y::{NodeGraphA11yFocusedEdge, NodeGraphA11yFocusedNode, NodeGraphA11yFocusedPort};
pub use canvas::NodeGraphCanvas;
pub use canvas::NodeGraphCanvasWith;
pub use canvas::NodeResizeHandle;
pub use canvas::{
    NodeGraphCanvasCommandOutcome, NodeGraphCanvasCommitOutcome, NodeGraphCanvasEventOutcome,
    NodeGraphCanvasMiddleware, NodeGraphCanvasMiddlewareChain, NodeGraphCanvasMiddlewareCx,
    NoopNodeGraphCanvasMiddleware,
};
pub use commands::register_node_graph_commands;
pub use edge_types::{EdgeCustomPath, EdgePathInput, EdgeTypeKey, NodeGraphEdgeTypes};
pub use edit_queue::NodeGraphEditQueue;
pub use editor::NodeGraphEditor;
pub use editors::{
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
    PortalTextEditorUi,
};
pub use internals::{
    NodeGraphCanvasTransform, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};
pub use measured::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter,
};
pub use panel::{NodeGraphPanel, NodeGraphPanelPosition, NodeGraphPanelSize};
pub use portal::{
    CMD_CANCEL_TEXT_PREFIX, CMD_STEP_TEXT_PREFIX, CMD_SUBMIT_TEXT_PREFIX,
    NodeGraphPortalCommandHandler, NodeGraphPortalHost, NodeGraphPortalNodeLayout,
    PortalCommandHandlerChain, PortalCommandOutcome, PortalNoopCommandHandler, PortalTextCommand,
    PortalTextStepMode, parse_portal_text_command, portal_cancel_text_command,
    portal_step_text_command, portal_step_text_command_with_mode, portal_submit_text_command,
};
pub use presenter::{
    DefaultNodeGraphPresenter, EdgeMarker, EdgeMarkerKind, EdgeRenderHint, EdgeRouteKind,
    InsertNodeCandidate, NodeGraphContextMenuAction, NodeGraphContextMenuItem, NodeGraphPresenter,
    NodeResizeConstraintsPx, NodeResizeHandleSet, RegistryNodeGraphPresenter,
};
pub use registry::{NodeGraphNodeRenderer, NodeGraphNodeTypes};
pub use style::{NodeGraphColorMode, NodeGraphStyle};
pub use view_queue::{
    NodeGraphFitViewOptions, NodeGraphSetViewportOptions, NodeGraphViewQueue, NodeGraphViewRequest,
};
pub use viewport_helper::NodeGraphViewportHelper;

pub use overlays::{
    GroupRenameOverlay, NodeGraphBlackboardOverlay, NodeGraphControlsBindings,
    NodeGraphControlsCommandBinding, NodeGraphControlsOverlay, NodeGraphEdgeToolbar,
    NodeGraphMiniMapBindings, NodeGraphMiniMapNavigationBinding, NodeGraphMiniMapOverlay,
    NodeGraphNodeToolbar, NodeGraphOverlayHost, NodeGraphOverlayState, NodeGraphToolbarAlign,
    NodeGraphToolbarPosition, NodeGraphToolbarSize, NodeGraphToolbarVisibility,
    SymbolRenameOverlay,
};
