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
    pub(crate) started: bool,
    pub(crate) dragging: bool,
    pub(crate) stopped: bool,
    pub(crate) delta: Point,
    pub(crate) total: Point,
}

/// Published state for an immediate drag source helper.
#[derive(Debug, Clone, Copy)]
pub struct DragSourceResponse {
    pub(crate) active: bool,
    pub(crate) cross_window: bool,
    pub(crate) position: Option<Point>,
    pub(crate) pointer_id: Option<PointerId>,
    pub(crate) session_id: Option<DragSessionId>,
}

/// Immediate drag/drop target readout for a typed payload.
pub struct DropTargetResponse<T: 'static> {
    pub(crate) active: bool,
    pub(crate) over: bool,
    pub(crate) delivered: bool,
    pub(crate) source_id: Option<GlobalElementId>,
    pub(crate) session_id: Option<fret_runtime::DragSessionId>,
    pub(in super::super) preview_position: Option<Point>,
    pub(in super::super) delivered_position: Option<Point>,
    pub(in super::super) preview_payload: Option<Rc<T>>,
    pub(in super::super) delivered_payload: Option<Rc<T>>,
}

impl DragSourceResponse {
    pub(crate) fn inactive() -> Self {
        Self {
            active: false,
            cross_window: false,
            position: None,
            pointer_id: None,
            session_id: None,
        }
    }

    pub(crate) fn new(
        cross_window: bool,
        position: Point,
        pointer_id: PointerId,
        session_id: DragSessionId,
    ) -> Self {
        Self {
            active: true,
            cross_window,
            position: Some(position),
            pointer_id: Some(pointer_id),
            session_id: Some(session_id),
        }
    }

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
    pub(crate) fn clear(&mut self) {
        self.dragging = false;
        self.delta = Point::default();
        self.total = Point::default();
    }

    pub(crate) fn set_started(&mut self, started: bool) {
        self.started = started;
    }

    pub(crate) fn set_dragging(&mut self, dragging: bool) {
        self.dragging = dragging;
    }

    pub(crate) fn set_stopped(&mut self, stopped: bool) {
        self.stopped = stopped;
    }

    pub(crate) fn set_motion(&mut self, delta: Point, total: Point) {
        self.delta = delta;
        self.total = total;
    }

    pub(crate) fn merge_edges(&mut self, other: Self) {
        self.started |= other.started;
        self.stopped |= other.stopped;
    }

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

impl<T: 'static> DropTargetResponse<T> {
    pub(crate) fn empty() -> Self {
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
