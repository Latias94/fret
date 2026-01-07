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
pub mod io;
pub mod ops;
pub mod rules;
pub mod schema;
pub mod types;

#[cfg(feature = "fret-ui")]
pub mod ui;

pub use core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Group, GroupId,
    Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
    StickyNote, StickyNoteId, Symbol, SymbolId,
};
pub use rules::{ConnectPlan, Diagnostic, DiagnosticSeverity};
pub use types::{TypeDesc, TypeVarId};
