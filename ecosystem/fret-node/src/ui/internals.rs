//! Derived editor internals output (UI-only).
//!
//! This module mirrors the "internals" concept from XyFlow/ReactFlow: the canonical graph model
//! stays pure data, while derived geometry (node rects, handle bounds, transforms) can be surfaced
//! for editor tooling (overlays, inspectors, debugging) without serializing it into assets.

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, EdgeId, NodeId, PortId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeGraphCanvasTransform {
    pub bounds_origin: Point,
    pub pan: CanvasPoint,
    pub zoom: f32,
}

impl Default for NodeGraphCanvasTransform {
    fn default() -> Self {
        Self {
            bounds_origin: Point::new(Px(0.0), Px(0.0)),
            pan: CanvasPoint::default(),
            zoom: 1.0,
        }
    }
}

impl NodeGraphCanvasTransform {
    pub fn canvas_point_to_window(self, p: Point) -> Point {
        let z = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        Point::new(
            Px(self.bounds_origin.x.0 + (p.x.0 + self.pan.x) * z),
            Px(self.bounds_origin.y.0 + (p.y.0 + self.pan.y) * z),
        )
    }

    pub fn canvas_rect_to_window(self, r: Rect) -> Rect {
        let z = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        let origin = Point::new(
            Px(self.bounds_origin.x.0 + (r.origin.x.0 + self.pan.x) * z),
            Px(self.bounds_origin.y.0 + (r.origin.y.0 + self.pan.y) * z),
        );
        let size = Size::new(Px(r.size.width.0 * z), Px(r.size.height.0 * z));
        Rect::new(origin, size)
    }
}

#[derive(Debug, Clone, Default)]
pub struct NodeGraphInternalsSnapshot {
    pub transform: NodeGraphCanvasTransform,
    pub nodes_window: BTreeMap<NodeId, Rect>,
    pub ports_window: BTreeMap<PortId, Rect>,
    pub port_centers_window: BTreeMap<PortId, Point>,
    /// Optional human-readable label for the currently active descendant (a11y support).
    ///
    /// This is an editor-derived surface and must not be serialized into graph assets.
    pub a11y_active_descendant_label: Option<String>,
    pub a11y_focused_node_label: Option<String>,
    pub a11y_focused_port_label: Option<String>,
    pub a11y_focused_edge_label: Option<String>,
    pub focused_node: Option<NodeId>,
    pub focused_port: Option<PortId>,
    pub focused_edge: Option<EdgeId>,
    pub connecting: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NodeGraphA11ySnapshot {
    pub active_descendant_label: Option<String>,
    pub focused_node_label: Option<String>,
    pub focused_port_label: Option<String>,
    pub focused_edge_label: Option<String>,
    pub focused_node: Option<NodeId>,
    pub focused_port: Option<PortId>,
    pub focused_edge: Option<EdgeId>,
    pub connecting: bool,
}

#[derive(Debug, Default)]
pub struct NodeGraphInternalsStore {
    revision: AtomicU64,
    snapshot: RwLock<NodeGraphInternalsSnapshot>,
}

impl NodeGraphInternalsStore {
    pub fn new() -> Self {
        Self {
            revision: AtomicU64::new(1),
            snapshot: RwLock::new(NodeGraphInternalsSnapshot::default()),
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> NodeGraphInternalsSnapshot {
        self.snapshot.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn a11y_snapshot(&self) -> NodeGraphA11ySnapshot {
        self.snapshot
            .read()
            .map(|s| NodeGraphA11ySnapshot {
                active_descendant_label: s.a11y_active_descendant_label.clone(),
                focused_node_label: s.a11y_focused_node_label.clone(),
                focused_port_label: s.a11y_focused_port_label.clone(),
                focused_edge_label: s.a11y_focused_edge_label.clone(),
                focused_node: s.focused_node,
                focused_port: s.focused_port,
                focused_edge: s.focused_edge,
                connecting: s.connecting,
            })
            .unwrap_or_default()
    }

    pub fn update(&self, next: NodeGraphInternalsSnapshot) -> u64 {
        if let Ok(mut s) = self.snapshot.write() {
            *s = next;
        }
        let old = self.revision.fetch_add(1, Ordering::Relaxed);
        old.wrapping_add(1)
    }
}
