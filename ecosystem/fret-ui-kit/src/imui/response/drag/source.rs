use fret_core::{Point, PointerId};
use fret_runtime::DragSessionId;

/// Published state for an immediate drag source helper.
#[derive(Debug, Clone, Copy)]
pub struct DragSourceResponse {
    pub(crate) active: bool,
    pub(crate) cross_window: bool,
    pub(crate) position: Option<Point>,
    pub(crate) pointer_id: Option<PointerId>,
    pub(crate) session_id: Option<DragSessionId>,
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
