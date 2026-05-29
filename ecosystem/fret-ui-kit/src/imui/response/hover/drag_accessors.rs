use fret_core::Point;

use super::super::drag::DragResponse;
use super::ResponseExt;

impl ResponseExt {
    pub(crate) fn drag_mut(&mut self) -> &mut DragResponse {
        &mut self.drag
    }

    pub fn drag(self) -> DragResponse {
        self.drag
    }

    pub fn drag_started(self) -> bool {
        self.drag.started()
    }

    pub fn dragging(self) -> bool {
        self.drag.dragging()
    }

    pub fn drag_stopped(self) -> bool {
        self.drag.stopped()
    }

    pub fn drag_delta(self) -> Point {
        self.drag.delta()
    }

    pub fn drag_total(self) -> Point {
        self.drag.total()
    }
}
