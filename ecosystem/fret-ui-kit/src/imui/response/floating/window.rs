use fret_core::{Point, Rect, Size};
use fret_ui::GlobalElementId;

use super::FloatingAreaResponse;

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
