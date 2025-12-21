use std::any::Any;

use fret_core::{AppWindowId, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragKind {
    DockPanel,
    Custom,
}

#[derive(Debug)]
pub struct DragSession {
    pub source_window: AppWindowId,
    pub current_window: AppWindowId,
    pub cross_window_hover: bool,
    pub kind: DragKind,
    pub start: Point,
    pub position: Point,
    pub dragging: bool,
    payload: Box<dyn Any>,
}

impl DragSession {
    pub fn new<T: Any>(
        source_window: AppWindowId,
        kind: DragKind,
        start: Point,
        payload: T,
    ) -> Self {
        Self {
            source_window,
            current_window: source_window,
            cross_window_hover: false,
            kind,
            start,
            position: start,
            dragging: false,
            payload: Box::new(payload),
        }
    }

    pub fn new_cross_window<T: Any>(
        source_window: AppWindowId,
        kind: DragKind,
        start: Point,
        payload: T,
    ) -> Self {
        Self {
            source_window,
            current_window: source_window,
            cross_window_hover: true,
            kind,
            start,
            position: start,
            dragging: false,
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
