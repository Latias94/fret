//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.

pub mod canvas;
pub mod commands;
pub mod edit_queue;
pub mod editor;
pub mod internals;
pub mod measured;
pub mod overlays;
pub mod portal;
pub mod presenter;
pub mod style;

pub use canvas::NodeGraphCanvas;
pub use commands::register_node_graph_commands;
pub use edit_queue::NodeGraphEditQueue;
pub use editor::NodeGraphEditor;
pub use internals::{
    NodeGraphCanvasTransform, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};
pub use measured::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter,
};
pub use portal::{NodeGraphPortalHost, NodeGraphPortalNodeLayout};
pub use presenter::{
    DefaultNodeGraphPresenter, InsertNodeCandidate, NodeGraphContextMenuAction,
    NodeGraphContextMenuItem, NodeGraphPresenter, RegistryNodeGraphPresenter,
};
pub use style::NodeGraphStyle;

pub use overlays::{GroupRenameOverlay, NodeGraphOverlayHost, NodeGraphOverlayState};
