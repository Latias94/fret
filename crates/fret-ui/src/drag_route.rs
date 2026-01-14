//! Generic internal-drag routing override.
//!
//! Some cross-window drag flows need to route internal drag events to a specific "anchor" node,
//! even when hit-testing fails or the cursor is over an unrelated widget (e.g. docking tear-off
//! drags). This is a mechanism-only extension point: the runtime does not own the policy for any
//! particular drag kind.

use fret_core::{AppWindowId, NodeId};
use fret_runtime::DragKindId;
use std::collections::HashMap;

/// Per-window internal drag routing table (mechanism-only).
///
/// Component/app code can register an override target for a given drag kind, and the runtime will
/// use it as the dispatch root for internal drag events when appropriate.
#[derive(Debug, Default)]
pub struct InternalDragRouteService {
    routes: HashMap<(AppWindowId, DragKindId), NodeId>,
}

impl InternalDragRouteService {
    pub fn set(&mut self, window: AppWindowId, kind: DragKindId, node: NodeId) {
        self.routes.insert((window, kind), node);
    }

    pub fn remove(&mut self, window: AppWindowId, kind: DragKindId) {
        self.routes.remove(&(window, kind));
    }

    pub fn route(&self, window: AppWindowId, kind: DragKindId) -> Option<NodeId> {
        self.routes.get(&(window, kind)).copied()
    }

    pub fn clear_window(&mut self, window: AppWindowId) {
        self.routes.retain(|(w, _), _| *w != window);
    }
}
