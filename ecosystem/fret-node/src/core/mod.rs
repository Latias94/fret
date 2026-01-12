//! Core graph model types (IDs, nodes, ports, edges, symbols).

mod ids;
mod model;
mod validate;

pub use ids::{
    EdgeId, GraphId, GroupId, NodeId, NodeKindKey, PortId, PortKey, StickyNoteId, SymbolId,
};
pub use model::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeKind, EdgeReconnectable,
    EdgeReconnectableEndpoint, Graph, Group, Node, NodeExtent, Port, PortCapacity, PortDirection,
    PortKind, StickyNote, Symbol,
};
pub use validate::{GraphValidationError, GraphValidationReport, validate_graph};

#[cfg(test)]
mod tests;
