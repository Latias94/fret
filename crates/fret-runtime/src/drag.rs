use std::any::Any;

use fret_core::{AppWindowId, Point, PointerId};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragKindId(pub u64);

pub const DRAG_KIND_DOCK_PANEL: DragKindId = DragKindId(1);
pub const DRAG_KIND_DOCK_TABS: DragKindId = DragKindId(2);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragSessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragPhase {
    Starting,
    Dragging,
    Dropped,
    Canceled,
}

#[derive(Debug)]
pub struct DragSession {
    pub session_id: DragSessionId,
    pub pointer_id: PointerId,
    pub source_window: AppWindowId,
    pub current_window: AppWindowId,
    pub cross_window_hover: bool,
    pub kind: DragKindId,
    pub start_position: Point,
    pub position: Point,
    pub dragging: bool,
    pub phase: DragPhase,
    payload: Box<dyn Any>,
}

impl DragSession {
    pub fn new<T: Any>(
        session_id: DragSessionId,
        pointer_id: PointerId,
        source_window: AppWindowId,
        kind: DragKindId,
        start_position: Point,
        payload: T,
    ) -> Self {
        Self {
            session_id,
            pointer_id,
            source_window,
            current_window: source_window,
            cross_window_hover: false,
            kind,
            start_position,
            position: start_position,
            dragging: false,
            phase: DragPhase::Starting,
            payload: Box::new(payload),
        }
    }

    pub fn new_cross_window<T: Any>(
        session_id: DragSessionId,
        pointer_id: PointerId,
        source_window: AppWindowId,
        kind: DragKindId,
        start_position: Point,
        payload: T,
    ) -> Self {
        Self {
            session_id,
            pointer_id,
            source_window,
            current_window: source_window,
            cross_window_hover: true,
            kind,
            start_position,
            position: start_position,
            dragging: false,
            phase: DragPhase::Starting,
            payload: Box::new(payload),
        }
    }

    pub fn payload<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    pub fn payload_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.payload.downcast_mut::<T>()
    }

    pub fn into_payload(self) -> Box<dyn Any> {
        self.payload
    }
}
