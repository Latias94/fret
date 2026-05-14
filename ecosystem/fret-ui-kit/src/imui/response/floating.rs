use fret_core::{Point, Rect, Size};
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

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResponse {
    pub(crate) area: FloatingAreaResponse,
    pub(crate) size: Option<Size>,
    pub(crate) resizing: bool,
    pub(crate) collapsed: bool,
}

impl FloatingWindowResponse {
    pub fn area(self) -> FloatingAreaResponse {
        self.area
    }

    pub fn id(self) -> GlobalElementId {
        self.area.id()
    }

    pub fn rect(self) -> Option<Rect> {
        self.area.rect()
    }

    pub fn position(self) -> Point {
        self.area.position()
    }

    pub fn size(self) -> Option<Size> {
        self.size
    }

    pub fn dragging(self) -> bool {
        self.area.dragging()
    }

    pub fn drag_kind(self) -> fret_runtime::DragKindId {
        self.area.drag_kind()
    }

    pub fn resizing(self) -> bool {
        self.resizing
    }

    pub fn collapsed(self) -> bool {
        self.collapsed
    }
}
