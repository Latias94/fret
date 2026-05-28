use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::PressablePointerUpResult;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::store::{ActiveDragPayload, DeliveredDragPayload, ImUiDragDropStore};

pub(super) fn install_payload_lifecycle_hooks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
    store: Model<ImUiDragDropStore>,
    payload: Rc<dyn Any>,
) {
    let store_for_move = store.clone();
    let payload_for_move = payload.clone();
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

            let _ = host.update_model(&store_for_move, |st| {
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
                        payload: payload_for_move.clone(),
                    },
                );
            });
            false
        }),
    );

    let store_for_up = store;
    cx.pressable_add_on_pointer_up_for(
        trigger_id,
        Arc::new(move |host, _acx, up| {
            if up.button != MouseButton::Left {
                return PressablePointerUpResult::Continue;
            }

            let Some((session_id, hovered_target, payload)) = host
                .models_mut()
                .read(&store_for_up, |st| {
                    st.active.iter().find_map(|(session_id, active)| {
                        (active.pointer_id == up.pointer_id
                            && active.kind == kind
                            && active.source_id == trigger_id)
                            .then(|| (*session_id, active.hovered_target, active.payload.clone()))
                    })
                })
                .ok()
                .flatten()
            else {
                return PressablePointerUpResult::Continue;
            };

            let Some(target_id) = hovered_target else {
                return PressablePointerUpResult::Continue;
            };

            let _ = host.update_model(&store_for_up, |st| {
                st.delivered.insert(
                    target_id,
                    DeliveredDragPayload {
                        tick_id: up.tick_id,
                        session_id,
                        source_id: trigger_id,
                        position: up.position,
                        payload,
                    },
                );
            });

            PressablePointerUpResult::Continue
        }),
    );
}
