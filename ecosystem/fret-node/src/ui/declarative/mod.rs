//! Declarative authoring surfaces for the node graph UI.
//!
//! This module is intentionally **declarative-first**. When needed, it can host narrowly scoped
//! retained subtrees as an internal compatibility strategy (opt-in at the crate integration level),
//! but downstream authors should not need to touch `UiTree`/`Widget` or `retained_bridge::*`.

mod paint_only;
mod view_reducer;
pub use super::binding::NodeGraphSurfaceBinding;
pub use paint_only::{NodeGraphSurfaceProps, node_graph_surface};

#[cfg(feature = "compat-retained-canvas")]
mod compat_retained;

#[cfg(feature = "compat-retained-canvas")]
pub use compat_retained::{
    NodeGraphSurfaceCompatRetainedProps, node_graph_surface_compat_retained,
};
