//! Styling/skin layer for `fret-node` node graph UIs.
//!
//! This module defines a UI-only "skin" surface that can provide per-node/per-edge visual hints
//! without mutating or serializing into the `Graph` document.

use std::sync::Arc;

use fret_core::Color;
use fret_core::scene::DashPatternV1;

use crate::core::{EdgeId, Graph, NodeId};

use super::presenter::EdgeRenderHint;
use super::style::NodeGraphStyle;

/// Per-node chrome overrides (UI-only).
///
/// v1 is intentionally minimal and paint-only. Layout-affecting knobs should be added only with
/// explicit invalidation/caching contracts and conformance tests.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NodeChromeHint {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_selected: Option<Color>,
}

/// Per-edge chrome overrides (UI-only).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct EdgeChromeHint {
    pub color: Option<Color>,
    pub width_mul: Option<f32>,
    pub dash: Option<DashPatternV1>,
}

/// Skin resolver for node graph visuals.
///
/// This is UI-only policy: it must not be serialized into the graph document.
pub trait NodeGraphSkin: Send + Sync {
    /// Revision that invalidates paint caches.
    ///
    /// Implementations should bump this when any paint-relevant output changes. v1 does not
    /// define a geometry-affecting contract; changes must be paint-only.
    fn revision(&self) -> u64 {
        0
    }

    fn node_chrome_hint(
        &self,
        _graph: &Graph,
        _node: NodeId,
        _style: &NodeGraphStyle,
        _selected: bool,
    ) -> NodeChromeHint {
        NodeChromeHint::default()
    }

    /// Refines an edge render hint after presenter + `edgeTypes` resolution.
    ///
    /// v1 contract: refinements must be paint-only (color/width/dash) and must not affect hit
    /// testing beyond interaction widths.
    fn edge_render_hint(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &NodeGraphStyle,
        base: &EdgeRenderHint,
        selected: bool,
        hovered: bool,
    ) -> EdgeRenderHint {
        let _ = (graph, edge, style, selected, hovered);
        base.clone()
    }
}

#[derive(Debug, Default)]
pub struct NoopNodeGraphSkin;

impl NodeGraphSkin for NoopNodeGraphSkin {}

pub type NodeGraphSkinRef = Arc<dyn NodeGraphSkin>;
