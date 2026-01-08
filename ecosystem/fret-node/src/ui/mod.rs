//! Fret UI integration for the node graph editor.
//!
//! This module is behind the default `fret-ui` feature.

#![cfg(feature = "fret-ui")]

pub mod canvas;
pub mod commands;
pub mod edit_queue;
pub mod internals;
pub mod measured;
pub mod presenter;
pub mod style;

pub use canvas::NodeGraphCanvas;
pub use commands::register_node_graph_commands;
pub use edit_queue::NodeGraphEditQueue;
pub use internals::{
    NodeGraphCanvasTransform, NodeGraphInternalsSnapshot, NodeGraphInternalsStore,
};
pub use measured::{
    FallbackMeasuredNodeGraphPresenter, MeasuredGeometryStore, MeasuredNodeGraphPresenter,
};
pub use presenter::{
    DefaultNodeGraphPresenter, InsertNodeCandidate, NodeGraphContextMenuAction,
    NodeGraphContextMenuItem, NodeGraphPresenter, RegistryNodeGraphPresenter,
};
pub use style::NodeGraphStyle;
