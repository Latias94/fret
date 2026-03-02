use crate::core::{EdgeId, NodeId};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NodeGeometryOverrideV1 {
    pub size_px: Option<(f32, f32)>,
}

impl NodeGeometryOverrideV1 {
    pub fn normalized(mut self) -> Self {
        if let Some((w, h)) = self.size_px {
            if !w.is_finite() || !h.is_finite() || w <= 0.0 || h <= 0.0 {
                self.size_px = None;
            }
        }
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct EdgeGeometryOverrideV1 {
    /// Screen-space interaction width override in logical px.
    pub interaction_width_px: Option<f32>,
}

impl EdgeGeometryOverrideV1 {
    pub fn normalized(mut self) -> Self {
        if let Some(w) = self.interaction_width_px {
            if !w.is_finite() || w < 0.0 {
                self.interaction_width_px = None;
            }
        }
        self
    }
}

/// UI-only per-entity geometry overrides for the node graph.
///
/// This surface is geometry-affecting: changes must bump `revision()` so derived geometry and
/// hit-testing caches rebuild deterministically (see ADR 0308).
pub trait NodeGraphGeometryOverrides: Send + Sync {
    /// Revision that invalidates derived geometry + hit-testing caches.
    fn revision(&self) -> u64;

    fn node_geometry_override(&self, _node: NodeId) -> NodeGeometryOverrideV1 {
        NodeGeometryOverrideV1::default()
    }

    fn edge_geometry_override(&self, _edge: EdgeId) -> EdgeGeometryOverrideV1 {
        EdgeGeometryOverrideV1::default()
    }

    /// Returns a conservative upper bound of `edge_geometry_override(...).interaction_width_px`
    /// across all overrides.
    ///
    /// Used to ensure the spatial index remains conservative for hit-testing.
    fn max_edge_interaction_width_override_px(&self) -> f32 {
        0.0
    }
}

pub type NodeGraphGeometryOverridesRef = Arc<dyn NodeGraphGeometryOverrides>;

/// A simple in-memory overrides map with explicit revision invalidation.
///
/// This is intended as a convenience surface for apps that want XyFlow-like per-entity geometry
/// overrides without implementing a custom provider.
pub struct NodeGraphGeometryOverridesMap {
    revision: AtomicU64,
    nodes: RwLock<BTreeMap<NodeId, NodeGeometryOverrideV1>>,
    edges: RwLock<BTreeMap<EdgeId, EdgeGeometryOverrideV1>>,
    max_edge_interaction_width_bits: AtomicU32,
}

impl Default for NodeGraphGeometryOverridesMap {
    fn default() -> Self {
        Self {
            revision: AtomicU64::new(1),
            nodes: RwLock::new(BTreeMap::new()),
            edges: RwLock::new(BTreeMap::new()),
            max_edge_interaction_width_bits: AtomicU32::new(0.0_f32.to_bits()),
        }
    }
}

impl NodeGraphGeometryOverridesMap {
    pub fn bump_revision(&self) {
        let _ = self.revision.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_node_size_px(&self, node: NodeId, size_px: (f32, f32)) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(
            node,
            NodeGeometryOverrideV1 {
                size_px: Some(size_px),
            }
            .normalized(),
        );
        drop(nodes);
        self.bump_revision();
    }

    pub fn clear_node_override(&self, node: NodeId) {
        let mut nodes = self.nodes.write().unwrap();
        let changed = nodes.remove(&node).is_some();
        drop(nodes);
        if changed {
            self.bump_revision();
        }
    }

    pub fn set_edge_interaction_width_px(&self, edge: EdgeId, width_px: f32) {
        let mut edges = self.edges.write().unwrap();
        edges.insert(
            edge,
            EdgeGeometryOverrideV1 {
                interaction_width_px: Some(width_px),
            }
            .normalized(),
        );
        self.refresh_max_edge_interaction_width_locked(&edges);
        drop(edges);
        self.bump_revision();
    }

    pub fn clear_edge_override(&self, edge: EdgeId) {
        let mut edges = self.edges.write().unwrap();
        let changed = edges.remove(&edge).is_some();
        if changed {
            self.refresh_max_edge_interaction_width_locked(&edges);
        }
        drop(edges);
        if changed {
            self.bump_revision();
        }
    }

    fn refresh_max_edge_interaction_width_locked(
        &self,
        edges: &BTreeMap<EdgeId, EdgeGeometryOverrideV1>,
    ) {
        let mut max_w = 0.0_f32;
        for o in edges.values() {
            if let Some(w) = o.interaction_width_px {
                if w.is_finite() && w >= 0.0 {
                    max_w = max_w.max(w);
                }
            }
        }
        self.max_edge_interaction_width_bits
            .store(max_w.to_bits(), Ordering::Relaxed);
    }
}

impl NodeGraphGeometryOverrides for NodeGraphGeometryOverridesMap {
    fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    fn node_geometry_override(&self, node: NodeId) -> NodeGeometryOverrideV1 {
        self.nodes
            .read()
            .unwrap()
            .get(&node)
            .copied()
            .unwrap_or_default()
            .normalized()
    }

    fn edge_geometry_override(&self, edge: EdgeId) -> EdgeGeometryOverrideV1 {
        self.edges
            .read()
            .unwrap()
            .get(&edge)
            .copied()
            .unwrap_or_default()
            .normalized()
    }

    fn max_edge_interaction_width_override_px(&self) -> f32 {
        f32::from_bits(self.max_edge_interaction_width_bits.load(Ordering::Relaxed))
    }
}
