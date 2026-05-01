use std::rc::Rc;

use fret_core::{Point, PointerId};
use fret_runtime::DragSessionId;
use fret_ui::GlobalElementId;

/// A richer interaction result intended for immediate-mode facade helpers.
///
/// This is a ui-kit-level convenience wrapper: it extends the minimal `fret-authoring::Response`
/// contract with additional commonly requested signals.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragResponse {
    pub started: bool,
    pub dragging: bool,
    pub stopped: bool,
    pub delta: Point,
    pub total: Point,
}

/// Published state for an immediate drag source helper.
#[derive(Debug, Clone, Copy, Default)]
pub struct DragSourceResponse {
    pub active: bool,
    pub cross_window: bool,
    pub position: Option<Point>,
    pub pointer_id: Option<PointerId>,
    pub session_id: Option<DragSessionId>,
}

/// Immediate drag/drop target readout for a typed payload.
pub struct DropTargetResponse<T: 'static> {
    pub active: bool,
    pub over: bool,
    pub delivered: bool,
    pub source_id: Option<GlobalElementId>,
    pub session_id: Option<fret_runtime::DragSessionId>,
    pub(in super::super) preview_position: Option<Point>,
    pub(in super::super) delivered_position: Option<Point>,
    pub(in super::super) preview_payload: Option<Rc<T>>,
    pub(in super::super) delivered_payload: Option<Rc<T>>,
}

impl DragSourceResponse {
    pub fn active(self) -> bool {
        self.active
    }

    pub fn cross_window(self) -> bool {
        self.cross_window
    }

    pub fn position(self) -> Option<Point> {
        self.position
    }

    pub fn pointer_id(self) -> Option<PointerId> {
        self.pointer_id
    }

    pub fn session_id(self) -> Option<DragSessionId> {
        self.session_id
    }
}

impl DragResponse {
    pub fn started(self) -> bool {
        self.started
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }

    pub fn stopped(self) -> bool {
        self.stopped
    }

    pub fn delta(self) -> Point {
        self.delta
    }

    pub fn total(self) -> Point {
        self.total
    }
}

impl<T: 'static> Default for DropTargetResponse<T> {
    fn default() -> Self {
        Self {
            active: false,
            over: false,
            delivered: false,
            source_id: None,
            session_id: None,
            preview_position: None,
            delivered_position: None,
            preview_payload: None,
            delivered_payload: None,
        }
    }
}

impl<T: 'static> DropTargetResponse<T> {
    pub fn active(&self) -> bool {
        self.active
    }

    pub fn over(&self) -> bool {
        self.over
    }

    pub fn delivered(&self) -> bool {
        self.delivered
    }

    pub fn preview_payload(&self) -> Option<Rc<T>> {
        self.preview_payload.clone()
    }

    pub fn preview_position(&self) -> Option<Point> {
        self.preview_position
    }

    pub fn delivered_payload(&self) -> Option<Rc<T>> {
        self.delivered_payload.clone()
    }

    pub fn delivered_position(&self) -> Option<Point> {
        self.delivered_position
    }

    pub fn source_id(&self) -> Option<GlobalElementId> {
        self.source_id
    }

    pub fn session_id(&self) -> Option<fret_runtime::DragSessionId> {
        self.session_id
    }
}
