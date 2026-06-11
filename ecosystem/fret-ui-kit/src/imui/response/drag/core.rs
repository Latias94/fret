use fret_core::Point;

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
