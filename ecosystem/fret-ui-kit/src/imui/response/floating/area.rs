use fret_core::{Point, Rect};
use fret_ui::GlobalElementId;

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaResponse {
    pub(crate) id: GlobalElementId,
    pub(crate) rect: Option<Rect>,
    pub(crate) position: Point,
    pub(crate) dragging: bool,
    pub(crate) drag_kind: fret_runtime::DragKindId,
}

impl FloatingAreaResponse {
    pub fn id(self) -> GlobalElementId {
        self.id
    }

    pub fn rect(self) -> Option<Rect> {
        self.rect
    }

    pub fn position(self) -> Point {
        self.position
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }

    pub fn drag_kind(self) -> fret_runtime::DragKindId {
        self.drag_kind
    }
}
