use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::super::store::{DeliveredDragPayload, ImUiDragDropStore};

pub(super) fn install_payload_up_delivery_hook<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
    store: Model<ImUiDragDropStore>,
) {
    cx.pressable_add_on_pointer_up_for(
        trigger_id,
        Arc::new(move |host, _acx, up| {
            if up.button != MouseButton::Left {
                return PressablePointerUpResult::Continue;
            }

            let Some((session_id, hovered_target, payload)) = host
                .models_mut()
                .read(&store, |st| {
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

            let _ = host.update_model(&store, |st| {
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
