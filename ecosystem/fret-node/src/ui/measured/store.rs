//! Measured geometry storage.

use std::collections::BTreeMap;
use std::sync::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::core::{NodeId, PortId};
use crate::ui::presenter::PortAnchorHint;

pub const MEASURED_GEOMETRY_EPSILON_PX: f32 = 0.25;

/// Thread-safe store for measured geometry hints.
///
/// Stored values are in screen-space logical pixels (px), consistent with `PortAnchorHint`.
#[derive(Debug, Default)]
pub struct MeasuredGeometryStore {
    revision: AtomicU64,
    node_sizes_px: RwLock<BTreeMap<NodeId, (f32, f32)>>,
    port_anchors_px: RwLock<BTreeMap<PortId, PortAnchorHint>>,
}

impl MeasuredGeometryStore {
    pub fn new() -> Self {
        Self {
            revision: AtomicU64::new(1),
            node_sizes_px: RwLock::new(BTreeMap::new()),
            port_anchors_px: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    pub fn bump_revision(&self) -> u64 {
        let old = self.revision.fetch_add(1, Ordering::Relaxed);
        old.wrapping_add(1)
    }

    pub fn update(
        &self,
        f: impl FnOnce(&mut BTreeMap<NodeId, (f32, f32)>, &mut BTreeMap<PortId, PortAnchorHint>),
    ) -> u64 {
        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");
        f(&mut node_sizes, &mut anchors);
        self.bump_revision()
    }

    pub fn update_if_changed(
        &self,
        f: impl FnOnce(&mut BTreeMap<NodeId, (f32, f32)>, &mut BTreeMap<PortId, PortAnchorHint>) -> bool,
    ) -> Option<u64> {
        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");
        let changed = f(&mut node_sizes, &mut anchors);
        changed.then(|| self.bump_revision())
    }

    pub fn node_size_px(&self, node: NodeId) -> Option<(f32, f32)> {
        self.node_sizes_px
            .read()
            .ok()
            .and_then(|m| m.get(&node).copied())
    }

    pub fn port_anchor_px(&self, port: PortId) -> Option<PortAnchorHint> {
        self.port_anchors_px
            .read()
            .ok()
            .and_then(|m| m.get(&port).copied())
    }

    /// Applies a batch of geometry updates, returning a new revision if anything changed.
    ///
    /// This is intended as a stable "internals update" surface, similar to XyFlow's
    /// `updateNodeInternals` action: a caller can publish measured node sizes and port anchor
    /// bounds without mutating the graph model.
    pub fn apply_batch_if_changed(
        &self,
        batch: MeasuredGeometryBatch,
        opts: MeasuredGeometryApplyOptions,
    ) -> Option<u64> {
        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");

        let mut changed = false;

        for node in &batch.remove_nodes {
            if node_sizes.remove(node).is_some() {
                changed = true;
            }
        }
        for port in &batch.remove_ports {
            if anchors.remove(port).is_some() {
                changed = true;
            }
        }

        for (node, size) in &batch.node_sizes_px {
            let needs = match node_sizes.get(node) {
                Some(old) => {
                    (old.0 - size.0).abs() > opts.epsilon_px
                        || (old.1 - size.1).abs() > opts.epsilon_px
                }
                None => true,
            };
            if needs {
                node_sizes.insert(*node, *size);
                changed = true;
            }
        }

        for (port, hint) in &batch.port_anchors_px {
            let needs = match anchors.get(port) {
                Some(old) => {
                    (old.center.x.0 - hint.center.x.0).abs() > opts.epsilon_px
                        || (old.center.y.0 - hint.center.y.0).abs() > opts.epsilon_px
                        || (old.bounds.origin.x.0 - hint.bounds.origin.x.0).abs() > opts.epsilon_px
                        || (old.bounds.origin.y.0 - hint.bounds.origin.y.0).abs() > opts.epsilon_px
                        || (old.bounds.size.width.0 - hint.bounds.size.width.0).abs()
                            > opts.epsilon_px
                        || (old.bounds.size.height.0 - hint.bounds.size.height.0).abs()
                            > opts.epsilon_px
                }
                None => true,
            };
            if needs {
                anchors.insert(*port, *hint);
                changed = true;
            }
        }

        changed.then(|| self.bump_revision())
    }

    /// Applies a batch that is treated as the full source of truth for this store.
    ///
    /// Any existing node/port entries not present in the batch are removed.
    pub fn apply_exclusive_batch_if_changed(
        &self,
        batch: MeasuredGeometryExclusiveBatch,
        opts: MeasuredGeometryApplyOptions,
    ) -> Option<u64> {
        let keep_nodes: std::collections::BTreeSet<NodeId> =
            batch.node_sizes_px.iter().map(|(id, _)| *id).collect();
        let keep_ports: std::collections::BTreeSet<PortId> =
            batch.port_anchors_px.iter().map(|(id, _)| *id).collect();

        let mut node_sizes = self.node_sizes_px.write().expect("poisoned lock");
        let mut anchors = self.port_anchors_px.write().expect("poisoned lock");

        let mut changed = false;

        node_sizes.retain(|id, _| {
            let ok = keep_nodes.contains(id);
            if !ok {
                changed = true;
            }
            ok
        });
        anchors.retain(|id, _| {
            let ok = keep_ports.contains(id);
            if !ok {
                changed = true;
            }
            ok
        });

        for (node, size) in &batch.node_sizes_px {
            let needs = match node_sizes.get(node) {
                Some(old) => {
                    (old.0 - size.0).abs() > opts.epsilon_px
                        || (old.1 - size.1).abs() > opts.epsilon_px
                }
                None => true,
            };
            if needs {
                node_sizes.insert(*node, *size);
                changed = true;
            }
        }
        for (port, hint) in &batch.port_anchors_px {
            let needs = match anchors.get(port) {
                Some(old) => {
                    (old.center.x.0 - hint.center.x.0).abs() > opts.epsilon_px
                        || (old.center.y.0 - hint.center.y.0).abs() > opts.epsilon_px
                        || (old.bounds.origin.x.0 - hint.bounds.origin.x.0).abs() > opts.epsilon_px
                        || (old.bounds.origin.y.0 - hint.bounds.origin.y.0).abs() > opts.epsilon_px
                        || (old.bounds.size.width.0 - hint.bounds.size.width.0).abs()
                            > opts.epsilon_px
                        || (old.bounds.size.height.0 - hint.bounds.size.height.0).abs()
                            > opts.epsilon_px
                }
                None => true,
            };
            if needs {
                anchors.insert(*port, *hint);
                changed = true;
            }
        }

        changed.then(|| self.bump_revision())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeasuredGeometryBatch {
    pub node_sizes_px: Vec<(NodeId, (f32, f32))>,
    pub port_anchors_px: Vec<(PortId, PortAnchorHint)>,
    pub remove_nodes: Vec<NodeId>,
    pub remove_ports: Vec<PortId>,
}

#[derive(Debug, Clone, Default)]
pub struct MeasuredGeometryExclusiveBatch {
    pub node_sizes_px: Vec<(NodeId, (f32, f32))>,
    pub port_anchors_px: Vec<(PortId, PortAnchorHint)>,
}

#[derive(Debug, Clone, Copy)]
pub struct MeasuredGeometryApplyOptions {
    pub epsilon_px: f32,
}

impl Default for MeasuredGeometryApplyOptions {
    fn default() -> Self {
        Self {
            epsilon_px: MEASURED_GEOMETRY_EPSILON_PX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px, Rect, Size};

    fn make_hint(x: f32, y: f32) -> PortAnchorHint {
        PortAnchorHint {
            center: Point::new(Px(x), Px(y)),
            bounds: Rect::new(
                Point::new(Px(x - 2.0), Px(y - 2.0)),
                Size::new(Px(4.0), Px(4.0)),
            ),
        }
    }

    #[test]
    fn apply_batch_within_epsilon_does_not_bump_revision() {
        let store = MeasuredGeometryStore::new();
        let node = NodeId::new();
        let port = PortId::new();

        let r0 = store.revision();
        assert!(
            store
                .apply_batch_if_changed(
                    MeasuredGeometryBatch {
                        node_sizes_px: vec![(node, (100.0, 100.0))],
                        port_anchors_px: vec![(port, make_hint(10.0, 10.0))],
                        remove_nodes: Vec::new(),
                        remove_ports: Vec::new(),
                    },
                    MeasuredGeometryApplyOptions::default()
                )
                .is_some()
        );
        let r1 = store.revision();
        assert!(r1 > r0);

        assert!(
            store
                .apply_batch_if_changed(
                    MeasuredGeometryBatch {
                        node_sizes_px: vec![(
                            node,
                            (100.0 + MEASURED_GEOMETRY_EPSILON_PX * 0.49, 100.0)
                        )],
                        port_anchors_px: vec![(
                            port,
                            make_hint(10.0 + MEASURED_GEOMETRY_EPSILON_PX * 0.49, 10.0)
                        )],
                        remove_nodes: Vec::new(),
                        remove_ports: Vec::new(),
                    },
                    MeasuredGeometryApplyOptions::default()
                )
                .is_none()
        );
        assert_eq!(store.revision(), r1);
    }

    #[test]
    fn apply_batch_beyond_epsilon_bumps_revision() {
        let store = MeasuredGeometryStore::new();
        let node = NodeId::new();
        let port = PortId::new();

        let _ = store.apply_batch_if_changed(
            MeasuredGeometryBatch {
                node_sizes_px: vec![(node, (100.0, 100.0))],
                port_anchors_px: vec![(port, make_hint(10.0, 10.0))],
                remove_nodes: Vec::new(),
                remove_ports: Vec::new(),
            },
            MeasuredGeometryApplyOptions::default(),
        );
        let r1 = store.revision();

        assert!(
            store
                .apply_batch_if_changed(
                    MeasuredGeometryBatch {
                        node_sizes_px: vec![(
                            node,
                            (100.0 + MEASURED_GEOMETRY_EPSILON_PX * 1.01, 100.0)
                        )],
                        port_anchors_px: vec![(
                            port,
                            make_hint(10.0 + MEASURED_GEOMETRY_EPSILON_PX * 1.01, 10.0)
                        )],
                        remove_nodes: Vec::new(),
                        remove_ports: Vec::new(),
                    },
                    MeasuredGeometryApplyOptions::default()
                )
                .is_some()
        );
        assert!(store.revision() > r1);
    }

    #[test]
    fn apply_exclusive_removes_missing_entries() {
        let store = MeasuredGeometryStore::new();
        let node_a = NodeId::new();
        let node_b = NodeId::new();
        let port_a = PortId::new();
        let port_b = PortId::new();

        let _ = store.apply_exclusive_batch_if_changed(
            MeasuredGeometryExclusiveBatch {
                node_sizes_px: vec![(node_a, (100.0, 100.0)), (node_b, (200.0, 200.0))],
                port_anchors_px: vec![
                    (port_a, make_hint(10.0, 10.0)),
                    (port_b, make_hint(20.0, 20.0)),
                ],
            },
            MeasuredGeometryApplyOptions::default(),
        );
        assert!(store.node_size_px(node_a).is_some());
        assert!(store.node_size_px(node_b).is_some());
        assert!(store.port_anchor_px(port_a).is_some());
        assert!(store.port_anchor_px(port_b).is_some());

        let r1 = store.revision();
        assert!(
            store
                .apply_exclusive_batch_if_changed(
                    MeasuredGeometryExclusiveBatch {
                        node_sizes_px: vec![(node_a, (100.0, 100.0))],
                        port_anchors_px: vec![(port_a, make_hint(10.0, 10.0))],
                    },
                    MeasuredGeometryApplyOptions::default()
                )
                .is_some()
        );
        assert!(store.revision() > r1);

        assert!(store.node_size_px(node_a).is_some());
        assert!(store.node_size_px(node_b).is_none());
        assert!(store.port_anchor_px(port_a).is_some());
        assert!(store.port_anchor_px(port_b).is_none());
    }
}
