//! Core graph model types (IDs, nodes, ports, edges, symbols).

mod ids;
mod imports;
mod model;
mod validate;

pub use ids::{
    EdgeId, GraphId, GroupId, NodeId, NodeKindKey, PortId, PortKey, StickyNoteId, SymbolId,
};
pub use imports::{GraphImport, GraphImportClosure, GraphImportError, resolve_import_closure};
pub use model::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeKind, EdgeReconnectable,
    EdgeReconnectableEndpoint, Graph, Group, Node, NodeExtent, Port, PortCapacity, PortDirection,
    PortKind, StickyNote, Symbol,
};
pub use validate::{
    GraphValidationError, GraphValidationReport, validate_graph, validate_graph_structural,
};

#[cfg(test)]
mod tests;
