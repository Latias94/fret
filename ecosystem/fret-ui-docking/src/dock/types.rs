use fret_core::{
    AppWindowId, Axis, DockNodeId, DropZone, PanelKey, TextBlobId, TextMetrics,
    geometry::{Point, Rect},
};

#[derive(Debug, Clone)]
pub(super) struct DockPanelDragPayload {
    pub(super) panel: PanelKey,
    pub(super) grab_offset: Point,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: AppWindowId },
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DividerDragState {
    pub(super) split: DockNodeId,
    pub(super) axis: Axis,
    pub(super) bounds: Rect,
    pub(super) handle_ix: usize,
    pub(super) grab_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HoverTarget {
    pub(super) tabs: DockNodeId,
    pub(super) zone: DropZone,
    pub(super) insert_index: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PreparedTabTitle {
    pub(super) blob: TextBlobId,
    pub(super) metrics: TextMetrics,
    pub(super) title_hash: u64,
}
