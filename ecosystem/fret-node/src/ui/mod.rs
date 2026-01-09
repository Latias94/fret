//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.

pub mod canvas;
pub mod commands;
pub mod edit_queue;
pub mod editor;
pub mod editors;
pub mod internals;
pub mod measured;
pub mod overlays;
pub mod portal;
pub mod presenter;
pub mod style;

pub use canvas::NodeGraphCanvas;
pub use canvas::NodeResizeHandle;
pub use commands::register_node_graph_commands;
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
pub use style::NodeGraphStyle;

pub use overlays::{
    GroupRenameOverlay, NodeGraphControlsOverlay, NodeGraphMiniMapOverlay, NodeGraphOverlayHost,
    NodeGraphOverlayState,
};
