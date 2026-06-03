use fret_core::{DockNodeId, PanelKey, Point, PointerId, Rect};

use crate::dock::types::DividerDragState;

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativePressedTabClose {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) tabs: DockNodeId,
    pub(in crate::dock::declarative) index: usize,
    pub(in crate::dock::declarative) panel: PanelKey,
    pub(in crate::dock::declarative) start: Point,
}

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativePressedFloatingClose {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) floating: DockNodeId,
}

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativeFloatingDrag {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) floating: DockNodeId,
    pub(in crate::dock::declarative) grab_offset: Point,
    pub(in crate::dock::declarative) start_rect: Rect,
    pub(in crate::dock::declarative) start: Point,
    pub(in crate::dock::declarative) start_tick: fret_runtime::TickId,
    pub(in crate::dock::declarative) activated: bool,
    pub(in crate::dock::declarative) dock_previews_enabled: bool,
}

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativeDividerDrag {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) handle: DividerDragState,
    pub(in crate::dock::declarative) min_px: Vec<fret_core::Px>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::dock::declarative) struct DeclarativeFloatingHover {
    pub(in crate::dock::declarative) close: Option<DockNodeId>,
    pub(in crate::dock::declarative) title_bar: Option<DockNodeId>,
}

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativePendingDockDrag {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) start: Point,
    pub(in crate::dock::declarative) panel: PanelKey,
    pub(in crate::dock::declarative) grab_offset: Point,
    pub(in crate::dock::declarative) start_tick: fret_runtime::TickId,
}

#[derive(Debug, Clone)]
pub(in crate::dock::declarative) struct DeclarativePendingDockTabsDrag {
    pub(in crate::dock::declarative) pointer_id: PointerId,
    pub(in crate::dock::declarative) start: Point,
    pub(in crate::dock::declarative) tabs: DockNodeId,
    pub(in crate::dock::declarative) grab_offset: Point,
    pub(in crate::dock::declarative) start_tick: fret_runtime::TickId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::dock::declarative) struct DeclarativeTabHover {
    pub(in crate::dock::declarative) tab: Option<(DockNodeId, usize)>,
    pub(in crate::dock::declarative) tab_close: bool,
    pub(in crate::dock::declarative) overflow_button: Option<DockNodeId>,
}
