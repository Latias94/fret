//! Core graph model types (IDs, nodes, ports, edges, symbols).

mod ids;
mod imports;
mod model;
mod subgraph;
mod symbol_ref;
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
pub use subgraph::{
    SUBGRAPH_NODE_KIND, SubgraphBindingError, SubgraphNodeError, collect_subgraph_targets,
    is_subgraph_node, subgraph_node_data, subgraph_target_graph_id,
    validate_subgraph_targets_are_imported,
};
pub use symbol_ref::{
    SYMBOL_REF_NODE_KIND, SymbolRefBindingError, SymbolRefNodeError, collect_symbol_ref_targets,
    is_symbol_ref_node, symbol_ref_node_data, symbol_ref_target_symbol_id,
    validate_symbol_ref_targets_are_declared,
};
pub use validate::{
    GraphValidationError, GraphValidationReport, validate_graph, validate_graph_structural,
};

#[cfg(test)]
mod tests;
