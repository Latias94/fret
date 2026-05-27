//! Shared portal layout data types.
//!
//! These types intentionally do not depend on retained widget plumbing so they can be used by
//! declarative-first surfaces and registries.

use fret_core::Rect;

use crate::core::NodeId;

/// Layout information for a portal-rendered node subtree.
#[derive(Debug, Clone, Copy)]
pub struct NodeGraphPortalNodeLayout {
    /// Node id in the graph model.
    pub node: NodeId,
    /// Node bounds in window coordinates.
    pub node_window: Rect,
    /// Zoom factor for the canvas.
    pub zoom: f32,
}
