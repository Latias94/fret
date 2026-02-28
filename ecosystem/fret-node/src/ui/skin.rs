//! Styling/skin layer for `fret-node` node graph UIs.
//!
//! This module defines a UI-only "skin" surface that can provide per-node/per-edge visual hints
//! without mutating or serializing into the `Graph` document.

use std::sync::Arc;

use fret_core::Color;
use fret_core::scene::DashPatternV1;

use crate::core::{EdgeId, Graph, NodeId, PortId};

use super::presenter::EdgeRenderHint;
use super::style::NodeGraphStyle;

/// Canvas-level chrome overrides (UI-only).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct CanvasChromeHint {
    pub background: Option<Color>,
    pub grid_minor: Option<Color>,
    pub grid_major: Option<Color>,
    /// Grid stroke thickness in screen-space logical px.
    pub grid_line_width_px: Option<f32>,
}

/// Interaction-level chrome overrides (UI-only).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct InteractionChromeHint {
    pub hover: Option<Color>,
    pub invalid: Option<Color>,
    pub convertible: Option<Color>,
    pub preview_wire: Option<Color>,
    pub dash_preview: Option<DashPatternV1>,
    pub dash_invalid: Option<DashPatternV1>,
    pub dash_emphasis: Option<DashPatternV1>,
    /// Optional wire glow applied to selected edges (paint-only effect).
    pub wire_glow_selected: Option<WireGlowHint>,
    /// Optional wire glow applied to drag preview wires (paint-only effect).
    pub wire_glow_preview: Option<WireGlowHint>,
    /// Optional wire outline applied to selected edges (paint-only, drawn behind the core stroke).
    pub wire_outline_selected: Option<WireOutlineHint>,
    /// Optional wire outline applied to drag preview wires (paint-only, drawn behind the core stroke).
    pub wire_outline_preview: Option<WireOutlineHint>,
}

/// Paint-only wire glow parameters (screen-space logical px).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WireGlowHint {
    pub blur_radius_px: f32,
    pub downsample: u32,
    pub alpha_mul: f32,
}

/// Paint-only wire outline parameters (screen-space logical px).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WireOutlineHint {
    pub width_mul: f32,
    pub color: Color,
}

/// Per-node chrome overrides (UI-only).
///
/// v1 is intentionally minimal and paint-only. Layout-affecting knobs should be added only with
/// explicit invalidation/caching contracts and conformance tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeRingHint {
    pub color: Color,
    /// Ring stroke width in screen-space logical px.
    pub width: f32,
    /// Ring pad (expands rect outward) in screen-space logical px.
    pub pad: f32,
}

/// Optional node shadow/glow hint (paint-only).
///
/// These values are expressed in screen-space logical px and must be converted by the canvas paint
/// path so they remain stable under zoom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeShadowHint {
    pub offset_x_px: f32,
    pub offset_y_px: f32,
    pub blur_radius_px: f32,
    pub downsample: u32,
    pub color: Color,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct NodeChromeHint {
    pub background: Option<Color>,
    pub border: Option<Color>,
    pub border_selected: Option<Color>,
    /// Optional node header background color (title strip).
    pub header_background: Option<Color>,
    /// Optional node title text color.
    pub title_text: Option<Color>,
    /// Optional selected ring (paint-only overlay, drawn outside the node rect).
    pub ring_selected: Option<NodeRingHint>,
    /// Optional focused ring (paint-only overlay, drawn outside the node rect).
    pub ring_focused: Option<NodeRingHint>,
    /// Optional node shadow/glow (paint-only effect).
    pub shadow: Option<NodeShadowHint>,
}

/// Per-edge chrome overrides (UI-only).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct EdgeChromeHint {
    pub color: Option<Color>,
    pub width_mul: Option<f32>,
    pub dash: Option<DashPatternV1>,
}

/// Port shape hint.
///
/// v1 only guarantees `Circle` rendering; other variants may fall back to circle until a vector
/// path-based implementation is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortShapeHint {
    Circle,
    Diamond,
    Triangle,
}

impl Default for PortShapeHint {
    fn default() -> Self {
        Self::Circle
    }
}

/// Per-port chrome overrides (UI-only).
///
/// v1 is paint-only: it must not change hit-testing or derived geometry. Size/shape changes should
/// be implemented as paint-only (e.g. inner scaling) unless explicitly paired with geometry
/// invalidation contracts and conformance tests.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PortChromeHint {
    /// Optional port fill override (defaults to presenter color).
    pub fill: Option<Color>,
    /// Optional port stroke color (drawn as an overlay).
    pub stroke: Option<Color>,
    /// Optional port stroke width in screen-space logical px.
    pub stroke_width: Option<f32>,
    /// Optional paint-only inner scale factor (0..=1) applied to the port rect.
    pub inner_scale: Option<f32>,
    /// Optional port shape hint (v1 only guarantees circle).
    pub shape: Option<PortShapeHint>,
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

    fn canvas_chrome_hint(&self, _graph: &Graph, _style: &NodeGraphStyle) -> CanvasChromeHint {
        CanvasChromeHint::default()
    }

    fn interaction_chrome_hint(
        &self,
        _graph: &Graph,
        _style: &NodeGraphStyle,
    ) -> InteractionChromeHint {
        InteractionChromeHint::default()
    }

    fn node_chrome_hint_with_state(
        &self,
        graph: &Graph,
        node: NodeId,
        style: &NodeGraphStyle,
        selected: bool,
        focused: bool,
    ) -> NodeChromeHint {
        let _ = focused;
        self.node_chrome_hint(graph, node, style, selected)
    }

    fn port_chrome_hint(
        &self,
        _graph: &Graph,
        _port: PortId,
        _style: &NodeGraphStyle,
        _base_fill: Color,
    ) -> PortChromeHint {
        PortChromeHint::default()
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
