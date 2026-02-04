//! Node graph substrate for Fret.
//!
//! This crate provides a long-lived, serializable graph model with typed connections and
//! editor-grade contracts (migrations, diagnostics, deterministic persistence).
//!
//! UI integration is optional and lives behind the default `fret-ui` feature.

#![deny(unsafe_code)]

/// Reserved builtin node kind for a schema-less wire reroute node.
pub const REROUTE_KIND: &str = "fret.reroute";

pub mod core;
pub mod interaction;
pub mod io;
#[cfg(feature = "kit")]
pub mod kit;
pub mod ops;
pub mod profile;
pub mod rules;
pub mod runtime;
pub mod schema;
pub mod types;

#[cfg(feature = "imui")]
pub mod imui;
#[cfg(feature = "fret-ui")]
pub mod ui;

pub use core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    StickyNote, StickyNoteId, Symbol, SymbolId,
};
pub use interaction::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphModifierKey,
    NodeGraphZoomActivationKey,
};
pub use rules::{ConnectPlan, Diagnostic, DiagnosticSeverity};
pub use types::{TypeDesc, TypeVarId};

#[cfg(feature = "app-integration")]
mod app_integration;
#[cfg(feature = "app-integration")]
pub use app_integration::install;
#[cfg(feature = "app-integration")]
pub use app_integration::install_app;
