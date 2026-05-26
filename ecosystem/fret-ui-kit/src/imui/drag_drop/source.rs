use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::MouseButton;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};

use super::super::{DragSourceOptions, DragSourceResponse, ResponseExt, UiWriterImUiFacadeExt};
use super::store::{
    ActiveDragPayload, DeliveredDragPayload, prune_store, source_response_for, store_model_for,
};

pub(in crate::imui) fn drag_source_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
    T: Any,
>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: DragSourceOptions,
) -> DragSourceResponse {
    let Some(trigger_id) = trigger.id() else {
        return DragSourceResponse::inactive();
    };

    let payload: Rc<dyn Any> = Rc::new(payload);
    ui.with_cx_mut(|cx| {
        let store = store_model_for(cx);
        prune_store(cx, &store);

        let kind = super::super::drag_kind_for_element(trigger_id);

        if options.enabled {
            if options.cross_window {
                cx.pressable_add_on_pointer_down_for(
                    trigger_id,
                    Arc::new(move |host, acx, down| {
                        if down.button != MouseButton::Left {
                            return PressablePointerDownResult::Continue;
                        }

                        let Some(drag) = host.drag(down.pointer_id) else {
                            return PressablePointerDownResult::Continue;
                        };
                        if drag.kind != kind
                            || drag.source_window != acx.window
                            || drag.cross_window_hover
                        {
                            return PressablePointerDownResult::Continue;
                        }

                        host.cancel_drag(down.pointer_id);
                        host.begin_cross_window_drag_with_kind(
                            down.pointer_id,
                            kind,
                            acx.window,
                            down.position,
                        );
                        PressablePointerDownResult::Continue
                    }),
                );
            }

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

            let store_for_up = store.clone();
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
                                    .then(|| {
                                        (*session_id, active.hovered_target, active.payload.clone())
                                    })
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

        source_response_for(cx, &store, trigger_id, kind)
    })
}
