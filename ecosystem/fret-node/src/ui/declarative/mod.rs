//! Declarative authoring surfaces for the node graph UI.
//!
//! This module is intentionally **declarative-first**. Downstream authors should not need to touch
//! `UiTree`/`Widget`, `retained_bridge::*`, or retained subtree compatibility entry points.

pub use super::editors::{
    PortalNumberEditHandler, PortalNumberEditSpec, PortalNumberEditSubmit, PortalNumberEditor,
    PortalTextEditHandler, PortalTextEditSpec, PortalTextEditSubmit, PortalTextEditor,
};
mod paint_only;
mod view_reducer;
pub use super::binding::NodeGraphSurfaceBinding;
pub use paint_only::{
    NodeGraphDeclarativePortalCommandHandler, NodeGraphDeclarativePortalCommandHandlerRef,
    NodeGraphDeclarativePortalRenderer, NodeGraphDiagnosticsConfig, NodeGraphSurfaceProps,
    NodeGraphVisibleSubsetPortalConfig, PortalCommandOutcome, PortalTextCommand,
    PortalTextStepMode, node_graph_surface, node_graph_surface_in,
    node_graph_surface_with_portal_renderer, node_graph_surface_with_portal_renderer_in,
    parse_portal_text_command, portal_cancel_text_command, portal_step_text_command,
    portal_step_text_command_with_mode, portal_submit_text_command,
};
