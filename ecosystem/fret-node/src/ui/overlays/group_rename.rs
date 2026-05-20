//! Node graph overlay host state (UI-only).

use fret_core::Point;

use crate::core::{GroupId, SymbolId};

/// UI-only overlay state for a node graph editor instance.
#[derive(Debug, Default, Clone)]
pub struct NodeGraphOverlayState {
    pub group_rename: Option<GroupRenameOverlay>,
    pub symbol_rename: Option<SymbolRenameOverlay>,
}

#[derive(Debug, Clone)]
pub struct GroupRenameOverlay {
    pub group: GroupId,
    pub invoked_at_window: Point,
}

#[derive(Debug, Clone)]
pub struct SymbolRenameOverlay {
    pub symbol: SymbolId,
    pub invoked_at_window: Point,
}
