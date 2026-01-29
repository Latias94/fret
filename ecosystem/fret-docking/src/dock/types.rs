use fret_core::{
    AppWindowId, Axis, DockNodeId, DropZone, PanelKey, TextBlobId, TextMetrics,
    geometry::{Point, Rect},
};
use fret_runtime::{FrameId, TickId};

#[derive(Debug, Clone)]
pub(super) struct DockPanelDragPayload {
    pub(super) panel: PanelKey,
    pub(super) grab_offset: Point,
    pub(super) start_tick: TickId,
    pub(super) tear_off_requested: bool,
    pub(super) tear_off_oob_start_frame: Option<FrameId>,
    pub(super) dock_previews_enabled: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DockTabsDragPayload {
    pub(super) source_tabs: DockNodeId,
    pub(super) tabs: Vec<PanelKey>,
    pub(super) active: usize,
    pub(super) grab_offset: Point,
    pub(super) start_tick: TickId,
    pub(super) dock_previews_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum DockDropTarget {
    Dock(HoverTarget),
    Float { window: AppWindowId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DockDropHints {
    pub(super) root: DockNodeId,
    pub(super) leaf_tabs: DockNodeId,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum DockDropIntent {
    None,
    MovePanel {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    },
    MoveTabs {
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
        target_tabs: DockNodeId,
        zone: DropZone,
        insert_index: Option<usize>,
    },
    FloatPanelInWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        target_window: AppWindowId,
        rect: Rect,
    },
    FloatTabsInWindow {
        source_window: AppWindowId,
        source_tabs: DockNodeId,
        target_window: AppWindowId,
        rect: Rect,
    },
    RequestFloatPanelToNewWindow {
        source_window: AppWindowId,
        panel: PanelKey,
        anchor: Option<fret_core::WindowAnchor>,
    },
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
    pub(super) root: DockNodeId,
    pub(super) leaf_tabs: DockNodeId,
    pub(super) zone: DropZone,
    pub(super) insert_index: Option<usize>,
    pub(super) outer: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PreparedTabTitle {
    pub(super) blob: TextBlobId,
    pub(super) metrics: TextMetrics,
    pub(super) title_hash: u64,
}
