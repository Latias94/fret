//! Headless graph primitives for Jellyflow.
//!
//! This crate owns the portable graph document model, stable IDs, type descriptors, and interaction
//! policy value types shared by higher-level runtimes and UI adapters. It must stay free of Fret UI,
//! renderer, platform, and windowing dependencies.

#![deny(unsafe_code)]

pub mod core;
pub mod interaction;
pub mod types;

pub use core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, EdgeReconnectable,
    EdgeReconnectableEndpoint, Graph, GraphId, GraphImport, GraphImportClosure, GraphImportError,
    GraphValidationError, GraphValidationReport, Group, GroupId, Node, NodeExtent, NodeId,
    NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind, StickyNote,
    StickyNoteId, Symbol, SymbolId,
};
pub use interaction::{
    NodeGraphConnectionMode, NodeGraphDragHandleMode, NodeGraphModifierKey,
    NodeGraphZoomActivationKey,
};
pub use types::{
    DefaultTypeCompatibility, TypeCompatibility, TypeCompatibilityResult, TypeDesc, TypeVarId,
};

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_stays_free_of_ui_renderer_and_platform_dependencies() {
        let manifest = include_str!("../Cargo.toml");
        for forbidden in ["fret-ui", "fret-runtime", "fret-canvas", "wgpu", "winit"] {
            assert!(
                !manifest.contains(forbidden),
                "jellyflow-core must stay headless; forbidden dependency `{forbidden}` found",
            );
        }
    }
}
