use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use super::{NodeGraphA11ySnapshot, NodeGraphInternalsSnapshot};

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

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use crate::core::{CanvasPoint, EdgeId, NodeId, PortId};

    use super::super::{NodeGraphCanvasTransform, NodeGraphInternalsSnapshot};
    use super::NodeGraphInternalsStore;

    #[test]
    fn a11y_snapshot_mirrors_internals_snapshot_fields() {
        let store = NodeGraphInternalsStore::new();
        let node = NodeId::new();
        let port = PortId::new();
        let edge = EdgeId::new();

        let snapshot = NodeGraphInternalsSnapshot {
            a11y_active_descendant_label: Some("Active".to_string()),
            a11y_focused_node_label: Some("Node A".to_string()),
            a11y_focused_port_label: Some("Port A".to_string()),
            a11y_focused_edge_label: Some("Edge A".to_string()),
            focused_node: Some(node),
            focused_port: Some(port),
            focused_edge: Some(edge),
            connecting: true,
            ..NodeGraphInternalsSnapshot::default()
        };
        store.update(snapshot);

        let a11y = store.a11y_snapshot();
        assert_eq!(a11y.active_descendant_label.as_deref(), Some("Active"));
        assert_eq!(a11y.focused_node_label.as_deref(), Some("Node A"));
        assert_eq!(a11y.focused_port_label.as_deref(), Some("Port A"));
        assert_eq!(a11y.focused_edge_label.as_deref(), Some("Edge A"));
        assert_eq!(a11y.focused_node, Some(node));
        assert_eq!(a11y.focused_port, Some(port));
        assert_eq!(a11y.focused_edge, Some(edge));
        assert!(a11y.connecting);
    }

    #[test]
    fn canvas_transform_uses_identity_zoom_when_invalid() {
        let transform = NodeGraphCanvasTransform {
            bounds_origin: Point::new(Px(100.0), Px(200.0)),
            bounds_size: Size::new(Px(800.0), Px(600.0)),
            pan: CanvasPoint { x: 5.0, y: -3.0 },
            zoom: f32::NAN,
        };

        let p = Point::new(Px(10.0), Px(20.0));
        let got = transform.canvas_point_to_window(p);
        assert!((got.x.0 - 115.0).abs() <= 1.0e-6);
        assert!((got.y.0 - 217.0).abs() <= 1.0e-6);

        let r = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            Size::new(Px(30.0), Px(40.0)),
        );
        let got = transform.canvas_rect_to_window(r);
        assert!((got.origin.x.0 - 115.0).abs() <= 1.0e-6);
        assert!((got.origin.y.0 - 217.0).abs() <= 1.0e-6);
        assert!((got.size.width.0 - 30.0).abs() <= 1.0e-6);
        assert!((got.size.height.0 - 40.0).abs() <= 1.0e-6);
    }
}
