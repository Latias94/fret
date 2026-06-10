use std::rc::Rc;

use crate::imui::{DragSourceOptions, DragSourceResponse, DropTargetOptions, DropTargetResponse};

/// Insertion side for a sortable target row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortableInsertionSide {
    Before,
    After,
}

impl SortableInsertionSide {
    pub fn label(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::After => "after",
        }
    }
}

/// Recipe-level options for a sortable immediate row.
#[derive(Debug, Clone, Copy, Default)]
pub struct SortableRowOptions {
    pub drag_source: DragSourceOptions,
    pub drop_target: DropTargetOptions,
}

/// Typed preview/delivery signal for one sortable row.
#[derive(Debug, Clone)]
pub struct SortableRowSignal<T: 'static> {
    payload: Rc<T>,
    side: SortableInsertionSide,
}

impl<T: 'static> SortableRowSignal<T> {
    pub fn payload(&self) -> Rc<T> {
        self.payload.clone()
    }

    pub fn side(&self) -> SortableInsertionSide {
        self.side
    }
}

/// Combined source/target readout for an immediate sortable row.
pub struct SortableRowResponse<T: 'static> {
    source: DragSourceResponse,
    target: DropTargetResponse<T>,
    side: Option<SortableInsertionSide>,
}

impl<T: 'static> SortableRowResponse<T> {
    pub(super) fn new(
        source: DragSourceResponse,
        target: DropTargetResponse<T>,
        side: Option<SortableInsertionSide>,
    ) -> Self {
        Self {
            source,
            target,
            side,
        }
    }

    pub fn source(&self) -> DragSourceResponse {
        self.source
    }

    pub fn target(&self) -> &DropTargetResponse<T> {
        &self.target
    }

    pub fn side(&self) -> Option<SortableInsertionSide> {
        self.side
    }

    pub fn preview_reorder(&self) -> Option<SortableRowSignal<T>> {
        Some(SortableRowSignal {
            payload: self.target.preview_payload()?,
            side: self.side?,
        })
    }

    pub fn delivered_reorder(&self) -> Option<SortableRowSignal<T>> {
        Some(SortableRowSignal {
            payload: self.target.delivered_payload()?,
            side: self.side?,
        })
    }
}
