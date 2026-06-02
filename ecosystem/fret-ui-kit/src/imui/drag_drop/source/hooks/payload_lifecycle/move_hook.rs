use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use fret_runtime::{DragKindId, Model};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::store::{ActiveDragPayload, ImUiDragDropStore};

pub(super) fn install_payload_move_hook<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
    store: Model<ImUiDragDropStore>,
    payload: Rc<dyn Any>,
) {
    cx.pressable_add_on_pointer_move_for(
        trigger_id,
        Arc::new(move |host, acx, mv| {
            let Some(session_id) = host.drag(mv.pointer_id).and_then(|drag| {
                if drag.kind != kind || drag.source_window != acx.window || !drag.dragging {
                    return None;
                }
                Some(drag.session_id)
            }) else {
                return false;
            };

            let _ = host.update_model(&store, |st| {
                let hovered_target = st
                    .active
                    .get(&session_id)
                    .and_then(|active| active.hovered_target);
                st.active.insert(
                    session_id,
                    ActiveDragPayload {
                        pointer_id: mv.pointer_id,
                        kind,
                        source_id: trigger_id,
                        hovered_target,
                        payload: payload.clone(),
                    },
                );
            });
            false
        }),
    );
}
