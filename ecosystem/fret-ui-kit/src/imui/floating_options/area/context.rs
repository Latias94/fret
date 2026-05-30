use fret_core::Point;
use fret_ui::GlobalElementId;

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaContext {
    pub(crate) id: GlobalElementId,
    pub(crate) position: Point,
    pub(crate) drag_kind: fret_runtime::DragKindId,
}

impl FloatingAreaContext {
    pub fn id(self) -> GlobalElementId {
        self.id
    }

    pub fn position(self) -> Point {
        self.position
    }

    pub fn drag_kind(self) -> fret_runtime::DragKindId {
        self.drag_kind
    }
}
