use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_core::scene::{DashPatternV1, PaintBindingV1};

use crate::core::{EdgeId, NodeId};

/// UI-only per-node paint overrides.
///
/// Contract: must be paint-only; geometry/hit-testing must not depend on these values.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NodePaintOverrideV1 {
    pub body_background: Option<PaintBindingV1>,
    pub border_paint: Option<PaintBindingV1>,
    pub header_background: Option<PaintBindingV1>,
}

impl NodePaintOverrideV1 {
    pub fn normalized(self) -> Self {
        Self {
            body_background: self.body_background.map(|p| p.sanitize()),
            border_paint: self.border_paint.map(|p| p.sanitize()),
            header_background: self.header_background.map(|p| p.sanitize()),
        }
    }
}

/// UI-only per-edge paint overrides.
///
/// Contract: must be paint-only; geometry/hit-testing must not depend on these values.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgePaintOverrideV1 {
    pub stroke_paint: Option<PaintBindingV1>,
    pub stroke_width_mul: Option<f32>,
    pub dash: Option<DashPatternV1>,
}

impl EdgePaintOverrideV1 {
    pub fn normalized(mut self) -> Self {
        self.stroke_paint = self.stroke_paint.map(|p| p.sanitize());
        if let Some(w) = self.stroke_width_mul
            && (!w.is_finite() || w <= 0.0) {
                self.stroke_width_mul = None;
            }
        if let Some(p) = self.dash {
            let dash = p.dash.0;
            let gap = p.gap.0;
            let phase = p.phase.0;
            let period = dash + gap;
            if !dash.is_finite()
                || !gap.is_finite()
                || !phase.is_finite()
                || dash <= 0.0
                || period <= 0.0
            {
                self.dash = None;
            }
        }
        self
    }
}

/// UI-only per-entity paint override provider (ADR 0309).
pub trait NodeGraphPaintOverrides: Send + Sync {
    /// Monotonically increasing revision; used to invalidate paint caches.
    fn revision(&self) -> u64 {
        0
    }

    fn node_paint_override(&self, _node: NodeId) -> Option<NodePaintOverrideV1> {
        None
    }

    fn edge_paint_override(&self, _edge: EdgeId) -> Option<EdgePaintOverrideV1> {
        None
    }
}

pub type NodeGraphPaintOverridesRef = Arc<dyn NodeGraphPaintOverrides>;

#[derive(Debug, Default)]
struct NodeGraphPaintOverridesMapState {
    per_node: HashMap<NodeId, NodePaintOverrideV1>,
    per_edge: HashMap<EdgeId, EdgePaintOverrideV1>,
}

/// In-memory paint override map with an atomic revision counter.
///
/// This is designed for host apps that want XyFlow-style per-entity overrides without mutating
/// the serialized `Graph`.
#[derive(Debug, Default)]
pub struct NodeGraphPaintOverridesMap {
    state: RwLock<NodeGraphPaintOverridesMapState>,
    revision: AtomicU64,
}

impl NodeGraphPaintOverridesMap {
    pub fn bump_revision(&self) -> u64 {
        self.revision
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1)
    }

    pub fn clear(&self) {
        let mut st = self.state.write().expect("paint overrides lock poisoned");
        st.per_node.clear();
        st.per_edge.clear();
        drop(st);
        self.bump_revision();
    }

    pub fn set_node_override(&self, node: NodeId, override_v1: Option<NodePaintOverrideV1>) {
        let mut st = self.state.write().expect("paint overrides lock poisoned");
        match override_v1 {
            Some(v) => {
                st.per_node.insert(node, v.normalized());
            }
            None => {
                st.per_node.remove(&node);
            }
        }
        drop(st);
        self.bump_revision();
    }

    pub fn set_edge_override(&self, edge: EdgeId, override_v1: Option<EdgePaintOverrideV1>) {
        let mut st = self.state.write().expect("paint overrides lock poisoned");
        match override_v1 {
            Some(v) => {
                st.per_edge.insert(edge, v.normalized());
            }
            None => {
                st.per_edge.remove(&edge);
            }
        }
        drop(st);
        self.bump_revision();
    }
}

impl NodeGraphPaintOverrides for NodeGraphPaintOverridesMap {
    fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    fn node_paint_override(&self, node: NodeId) -> Option<NodePaintOverrideV1> {
        let st = self.state.read().expect("paint overrides lock poisoned");
        st.per_node.get(&node).copied()
    }

    fn edge_paint_override(&self, edge: EdgeId) -> Option<EdgePaintOverrideV1> {
        let st = self.state.read().expect("paint overrides lock poisoned");
        st.per_edge.get(&edge).copied()
    }
}
