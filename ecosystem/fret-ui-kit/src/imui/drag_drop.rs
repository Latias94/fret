use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::{MouseButton, Point, PointerId};
use fret_runtime::{DragKindId, DragSessionId, Model, TickId};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, UiHost};

use super::{
    DragSourceOptions, DragSourceResponse, DropTargetOptions, DropTargetResponse, ResponseExt,
    UiWriterImUiFacadeExt,
};

#[derive(Default)]
struct ImUiDragDropStoreGlobal {
    model: Option<Model<ImUiDragDropStore>>,
}

#[derive(Default)]
struct ImUiDragDropStore {
    active: HashMap<DragSessionId, ActiveDragPayload>,
    delivered: HashMap<GlobalElementId, DeliveredDragPayload>,
}

#[derive(Clone)]
struct ActiveDragPayload {
    pointer_id: PointerId,
    kind: DragKindId,
    source_id: GlobalElementId,
    hovered_target: Option<GlobalElementId>,
    payload: Rc<dyn Any>,
}

#[derive(Clone)]
struct DeliveredDragPayload {
    tick_id: TickId,
    session_id: DragSessionId,
    source_id: GlobalElementId,
    position: Point,
    payload: Rc<dyn Any>,
}

fn store_model_for<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<ImUiDragDropStore> {
    cx.app
        .with_global_mut_untracked(ImUiDragDropStoreGlobal::default, |st, app| {
            if let Some(model) = st.model.clone() {
                return model;
            }

            let model = app.models_mut().insert(ImUiDragDropStore::default());
            st.model = Some(model.clone());
            model
        })
}

fn prune_store<H: UiHost>(cx: &mut ElementContext<'_, H>, store: &Model<ImUiDragDropStore>) {
    let current_tick = cx.app.tick_id();
    let stale_sessions = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    app.drag(active.pointer_id)
                        .filter(|drag| drag.session_id == *session_id && drag.kind == active.kind)
                        .map(|_| None)
                        .unwrap_or(Some(*session_id))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in &stale_sessions {
            st.active.remove(session_id);
        }

        st.delivered
            .retain(|_, delivery| current_tick.0 <= delivery.tick_id.0.saturating_add(1));
    });
}

fn source_response_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
) -> DragSourceResponse {
    let pointer_id = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    if active.source_id != trigger_id || active.kind != kind {
                        return None;
                    }
                    let drag = app.drag(active.pointer_id)?;
                    if drag.session_id != *session_id {
                        return None;
                    }
                    Some(active.pointer_id)
                })
                .min_by_key(|pointer_id| pointer_id.0)
        })
        .ok()
        .flatten();

    let Some(pointer_id) = pointer_id else {
        return DragSourceResponse::default();
    };
    let Some(drag) = cx.app.drag(pointer_id) else {
        return DragSourceResponse::default();
    };

    DragSourceResponse {
        active: true,
        cross_window: drag.cross_window_hover,
        position: Some(drag.position),
        pointer_id: Some(pointer_id),
        session_id: Some(drag.session_id),
    }
}

fn first_active_payload_for<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
) -> Option<(DragSessionId, GlobalElementId, Point, Rc<T>)> {
    cx.read_model(store, Invalidation::Paint, |app, st| {
        st.active
            .iter()
            .filter_map(|(session_id, active)| {
                let drag = app.drag(active.pointer_id)?;
                if drag.session_id != *session_id || !drag.dragging {
                    return None;
                }
                let payload = active.payload.clone().downcast::<T>().ok()?;
                Some((
                    active.pointer_id,
                    drag.session_id,
                    active.source_id,
                    drag.position,
                    payload,
                ))
            })
            .min_by_key(|(pointer_id, _, _, _, _)| pointer_id.0)
            .map(|(_, session_id, source_id, position, payload)| {
                (session_id, source_id, position, payload)
            })
    })
    .ok()
    .flatten()
}

fn take_delivered_payload_for<T: Any, H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ImUiDragDropStore>,
    target_id: GlobalElementId,
) -> Option<(DragSessionId, GlobalElementId, Point, Rc<T>)> {
    let current_tick = cx.app.tick_id();
    cx.app
        .models_mut()
        .update(store, |st| {
            let delivered = st.delivered.remove(&target_id)?;
            if current_tick.0 > delivered.tick_id.0.saturating_add(1) {
                return None;
            }
            let payload = delivered.payload.downcast::<T>().ok()?;
            Some((
                delivered.session_id,
                delivered.source_id,
                delivered.position,
                payload,
            ))
        })
        .ok()
        .flatten()
}

pub(super) fn drag_source_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    payload: T,
    options: DragSourceOptions,
) -> DragSourceResponse {
    let Some(trigger_id) = trigger.id else {
        return DragSourceResponse::default();
    };

    let payload: Rc<dyn Any> = Rc::new(payload);
    ui.with_cx_mut(|cx| {
        let store = store_model_for(cx);
        prune_store(cx, &store);

        let kind = super::drag_kind_for_element(trigger_id);

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

pub(super) fn drop_target_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized, T: Any>(
    ui: &mut W,
    trigger: ResponseExt,
    options: DropTargetOptions,
) -> DropTargetResponse<T> {
    let Some(trigger_id) = trigger.id else {
        return DropTargetResponse::default();
    };

    ui.with_cx_mut(|cx| {
        let store = store_model_for(cx);
        prune_store(cx, &store);

        let mut response = DropTargetResponse::default();
        if !options.enabled {
            return response;
        }

        if let Some((session_id, source_id, position, payload)) =
            take_delivered_payload_for::<T, _>(cx, &store, trigger_id)
        {
            response.active = true;
            response.delivered = true;
            response.source_id = Some(source_id);
            response.session_id = Some(session_id);
            response.delivered_position = Some(position);
            response.delivered_payload = Some(payload);
        }

        if let Some((session_id, source_id, position, payload)) =
            first_active_payload_for::<T, _>(cx, &store)
        {
            response.active = true;
            if response.source_id.is_none() {
                response.source_id = Some(source_id);
            }
            if response.session_id.is_none() {
                response.session_id = Some(session_id);
            }
            response.preview_position = Some(position);
            let _ = cx.app.models_mut().update(&store, |st| {
                if let Some(active) = st.active.get_mut(&session_id) {
                    if trigger.pointer_hovered_raw {
                        active.hovered_target = Some(trigger_id);
                    } else if active.hovered_target == Some(trigger_id) {
                        active.hovered_target = None;
                    }
                }
            });
            if trigger.pointer_hovered_raw {
                response.over = true;
                response.preview_payload = Some(payload);
            }
        }

        response
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_authoring::UiWriter;
    use fret_ui::element::AnyElement;

    struct TestWriter<'cx, 'a, H: UiHost> {
        cx: &'cx mut ElementContext<'a, H>,
        out: &'cx mut Vec<AnyElement>,
    }

    impl<'cx, 'a, H: UiHost> UiWriter<H> for TestWriter<'cx, 'a, H> {
        fn with_cx_mut<R>(&mut self, f: impl FnOnce(&mut ElementContext<'_, H>) -> R) -> R {
            f(self.cx)
        }

        fn add(&mut self, element: AnyElement) {
            self.out.push(element);
        }
    }

    #[test]
    fn drag_source_returns_inactive_without_trigger_id() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };
                let response = drag_source_with_options(
                    &mut ui,
                    ResponseExt::default(),
                    42_u32,
                    DragSourceOptions::default(),
                );
                assert!(!response.active());
                assert!(out.is_empty());
            },
        );
    }

    #[test]
    fn drop_target_returns_empty_without_trigger_id() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |cx| {
                let mut out = Vec::new();
                let mut ui = TestWriter { cx, out: &mut out };
                let response = drop_target_with_options::<_, _, u32>(
                    &mut ui,
                    ResponseExt::default(),
                    DropTargetOptions::default(),
                );
                assert!(!response.active());
                assert!(!response.over());
                assert!(!response.delivered());
                assert!(response.preview_payload().is_none());
                assert!(response.delivered_payload().is_none());
                assert!(out.is_empty());
            },
        );
    }
}
