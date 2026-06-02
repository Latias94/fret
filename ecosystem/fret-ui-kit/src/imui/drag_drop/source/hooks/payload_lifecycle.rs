use std::any::Any;
use std::rc::Rc;

use fret_runtime::{DragKindId, Model};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::store::ImUiDragDropStore;

mod move_hook;
mod up_delivery;

pub(super) fn install_payload_lifecycle_hooks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
    store: Model<ImUiDragDropStore>,
    payload: Rc<dyn Any>,
) {
    move_hook::install_payload_move_hook(cx, trigger_id, kind, store.clone(), payload);
    up_delivery::install_payload_up_delivery_hook(cx, trigger_id, kind, store);
}
