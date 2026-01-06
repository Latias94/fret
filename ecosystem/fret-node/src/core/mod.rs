//! Core graph model types (IDs, nodes, ports, edges, symbols).

mod ids;
mod model;

pub use ids::{
    EdgeId, GraphId, GroupId, NodeId, NodeKindKey, PortId, PortKey, StickyNoteId, SymbolId,
};
pub use model::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeKind, Graph, Group, Node, Port, PortCapacity,
    PortDirection, PortKind, StickyNote, Symbol,
};
