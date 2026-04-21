use fret_core::{Point, Rect, Size};
use fret_ui::GlobalElementId;

#[derive(Debug, Clone, Copy)]
pub struct FloatingAreaResponse {
    pub id: GlobalElementId,
    pub rect: Option<Rect>,
    pub position: Point,
    pub dragging: bool,
    pub drag_kind: fret_runtime::DragKindId,
}

impl FloatingAreaResponse {
    pub fn rect(self) -> Option<Rect> {
        self.rect
    }

    pub fn position(self) -> Point {
        self.position
    }

    pub fn dragging(self) -> bool {
        self.dragging
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FloatingWindowResponse {
    pub area: FloatingAreaResponse,
    pub size: Option<Size>,
    pub resizing: bool,
    pub collapsed: bool,
}

impl FloatingWindowResponse {
    pub fn rect(self) -> Option<Rect> {
        self.area.rect
    }

    pub fn position(self) -> Point {
        self.area.position
    }

    pub fn size(self) -> Option<Size> {
        self.size
    }

    pub fn dragging(self) -> bool {
        self.area.dragging
    }

    pub fn resizing(self) -> bool {
        self.resizing
    }

    pub fn collapsed(self) -> bool {
        self.collapsed
    }
}
