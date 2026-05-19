//! Declarative authoring surfaces for the node graph UI.
//!
//! This module is intentionally **declarative-first**. Downstream authors should not need to touch
//! `UiTree`/`Widget`, `retained_bridge::*`, or retained subtree compatibility entry points.

mod paint_only;
mod view_reducer;
pub use super::binding::NodeGraphSurfaceBinding;
pub use paint_only::{
    NodeGraphDiagnosticsConfig, NodeGraphSurfaceProps, NodeGraphVisibleSubsetPortalConfig,
    node_graph_surface, node_graph_surface_in,
};
