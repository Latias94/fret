use std::collections::HashMap;
use std::sync::Arc;

use fret_ui::GlobalElementId;
use fret_ui::element::AnyElement;

#[derive(Debug, Default)]
pub(super) struct FloatWindowLayerZOrder {
    order: Vec<GlobalElementId>,
    dirty: bool,
    snapshot: FloatWindowLayerZOrderSnapshot,
}

impl FloatWindowLayerZOrder {
    pub(super) fn ensure_present(&mut self, window: GlobalElementId) {
        if self.order.contains(&window) {
            return;
        }
        self.order.push(window);
        self.dirty = true;
    }

    pub(super) fn bring_to_front(&mut self, window: GlobalElementId) {
        self.ensure_present(window);
        let Some(idx) = self.order.iter().position(|w| *w == window) else {
            return;
        };
        if idx + 1 == self.order.len() {
            return;
        }
        self.order.remove(idx);
        self.order.push(window);
        self.dirty = true;
    }

    pub(super) fn prune_missing(&mut self, windows: &[AnyElement]) {
        let before = self.order.len();
        self.order.retain(|id| windows.iter().any(|w| w.id == *id));
        if self.order.len() != before {
            self.dirty = true;
        }
    }

    pub(super) fn snapshot(&mut self) -> FloatWindowLayerZOrderSnapshot {
        if !self.dirty {
            return self.snapshot.clone();
        }

        let order: Arc<[GlobalElementId]> = self.order.clone().into();
        let mut rank = HashMap::with_capacity(order.len());
        for (ix, id) in order.iter().enumerate() {
            rank.insert(*id, ix);
        }

        self.snapshot = FloatWindowLayerZOrderSnapshot {
            order,
            rank: Arc::new(rank),
        };
        self.dirty = false;
        self.snapshot.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct FloatWindowLayerZOrderSnapshot {
    #[allow(dead_code)]
    order: Arc<[GlobalElementId]>,
    pub(super) rank: Arc<HashMap<GlobalElementId, usize>>,
}
