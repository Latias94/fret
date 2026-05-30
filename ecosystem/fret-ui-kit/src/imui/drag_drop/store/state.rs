use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use fret_core::{Point, PointerId};
use fret_runtime::{DragKindId, DragSessionId, Model, TickId};
use fret_ui::GlobalElementId;

#[derive(Default)]
pub(super) struct ImUiDragDropStoreGlobal {
    pub(super) model: Option<Model<ImUiDragDropStore>>,
}

#[derive(Default)]
pub(in crate::imui::drag_drop) struct ImUiDragDropStore {
    pub(in crate::imui::drag_drop) active: HashMap<DragSessionId, ActiveDragPayload>,
    pub(in crate::imui::drag_drop) delivered: HashMap<GlobalElementId, DeliveredDragPayload>,
}

#[derive(Clone)]
pub(in crate::imui::drag_drop) struct ActiveDragPayload {
    pub(in crate::imui::drag_drop) pointer_id: PointerId,
    pub(in crate::imui::drag_drop) kind: DragKindId,
    pub(in crate::imui::drag_drop) source_id: GlobalElementId,
    pub(in crate::imui::drag_drop) hovered_target: Option<GlobalElementId>,
    pub(in crate::imui::drag_drop) payload: Rc<dyn Any>,
}

#[derive(Clone)]
pub(in crate::imui::drag_drop) struct DeliveredDragPayload {
    pub(in crate::imui::drag_drop) tick_id: TickId,
    pub(in crate::imui::drag_drop) session_id: DragSessionId,
    pub(in crate::imui::drag_drop) source_id: GlobalElementId,
    pub(in crate::imui::drag_drop) position: Point,
    pub(in crate::imui::drag_drop) payload: Rc<dyn Any>,
}
