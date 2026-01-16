use std::sync::Arc;

use fret_core::PointerId;
use fret_runtime::DragKindId;

/// Drag kind for cross-pane workspace tab drags.
pub const DRAG_KIND_WORKSPACE_TAB: DragKindId = DragKindId(0x57535F544142); // "WS_TAB"

#[derive(Debug, Default, Clone)]
pub struct WorkspaceTabDragState {
    pub pointer: Option<PointerId>,
    pub source_pane: Option<Arc<str>>,
    pub dragged_tab: Option<Arc<str>>,
    pub hovered_pane: Option<Arc<str>>,
}
