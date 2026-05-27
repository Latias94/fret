use std::rc::Rc;

use fret_core::Point;
use fret_ui::GlobalElementId;

/// Immediate drag/drop target readout for a typed payload.
pub struct DropTargetResponse<T: 'static> {
    pub(crate) active: bool,
    pub(crate) over: bool,
    pub(crate) delivered: bool,
    pub(crate) source_id: Option<GlobalElementId>,
    pub(crate) session_id: Option<fret_runtime::DragSessionId>,
    pub(in crate::imui) preview_position: Option<Point>,
    pub(in crate::imui) delivered_position: Option<Point>,
    pub(in crate::imui) preview_payload: Option<Rc<T>>,
    pub(in crate::imui) delivered_payload: Option<Rc<T>>,
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
