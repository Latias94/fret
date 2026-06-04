use fret_core::{MouseButton, Point, Px};
use fret_runtime::{DragKindId, DragPhase, Model};
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::{ActiveColorDrag, ColorDragDropStore, DeliveredColorDrop};
use crate::controls::color_edit::{ColorEditDragDropOptions, ColorEditDragDropPayload};

const COLOR_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;

pub(in crate::controls::color_edit) fn install_color_drag_source<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source_id: GlobalElementId,
    store: Model<ColorDragDropStore>,
    payload: ColorEditDragDropPayload,
    options: ColorEditDragDropOptions,
    threshold: Px,
) {
    if !options.enabled {
        return;
    }

    let kind = color_drag_kind_for_element(source_id);
    cx.pressable_add_on_pointer_down({
        let store = store.clone();
        std::sync::Arc::new(move |host, action_cx, down| {
            if down.button != MouseButton::Left {
                return PressablePointerDownResult::Continue;
            }

            if host.drag(down.pointer_id).is_none() {
                if options.cross_window {
                    host.begin_cross_window_drag_with_kind(
                        down.pointer_id,
                        kind,
                        action_cx.window,
                        down.position,
                    );
                } else {
                    host.begin_drag_with_kind(
                        down.pointer_id,
                        kind,
                        action_cx.window,
                        down.position,
                    );
                }
            }

            let _ = host.update_model(&store, |st| {
                st.active.retain(|_, active| {
                    !(active.pointer_id == down.pointer_id
                        && active.kind == kind
                        && active.source_id == source_id)
                });
            });

            PressablePointerDownResult::Continue
        })
    });

    cx.pressable_add_on_pointer_move({
        let store = store.clone();
        std::sync::Arc::new(move |host, action_cx, mv| {
            enum MoveResult {
                Pending,
                Active(fret_runtime::DragSessionId),
                Canceled(fret_runtime::DragSessionId),
            }

            let result = {
                let Some(drag) = host.drag_mut(mv.pointer_id) else {
                    return false;
                };
                if drag.kind != kind || drag.source_window != action_cx.window {
                    return false;
                }

                drag.current_window = action_cx.window;
                drag.position = mv.position;

                if !mv.buttons.left {
                    drag.phase = DragPhase::Canceled;
                    MoveResult::Canceled(drag.session_id)
                } else if drag.dragging {
                    drag.phase = DragPhase::Dragging;
                    MoveResult::Active(drag.session_id)
                } else if color_drag_threshold_exceeded(
                    drag.start_position,
                    drag.position,
                    threshold,
                ) {
                    drag.dragging = true;
                    drag.phase = DragPhase::Dragging;
                    MoveResult::Active(drag.session_id)
                } else {
                    MoveResult::Pending
                }
            };

            match result {
                MoveResult::Pending => false,
                MoveResult::Active(session_id) => {
                    let _ = host.update_model(&store, |st| {
                        let hovered_target = st
                            .active
                            .get(&session_id)
                            .and_then(|active| active.hovered_target);
                        st.active.insert(
                            session_id,
                            ActiveColorDrag {
                                pointer_id: mv.pointer_id,
                                kind,
                                source_id,
                                hovered_target,
                                payload,
                            },
                        );
                    });
                    host.request_redraw(action_cx.window);
                    false
                }
                MoveResult::Canceled(session_id) => {
                    let _ = host.update_model(&store, |st| {
                        st.active.remove(&session_id);
                    });
                    host.cancel_drag(mv.pointer_id);
                    host.request_redraw(action_cx.window);
                    false
                }
            }
        })
    });

    cx.pressable_add_on_pointer_up({
        let store = store.clone();
        std::sync::Arc::new(move |host, action_cx, up| {
            if up.button != MouseButton::Left {
                return PressablePointerUpResult::Continue;
            }

            let was_dragging = host.drag(up.pointer_id).is_some_and(|drag| {
                drag.kind == kind && drag.source_window == action_cx.window && drag.dragging
            });

            let delivered_or_active = host
                .update_model(&store, |st| {
                    let session_id = st.active.iter().find_map(|(session_id, active)| {
                        (active.pointer_id == up.pointer_id
                            && active.kind == kind
                            && active.source_id == source_id)
                            .then_some(*session_id)
                    });

                    let Some(session_id) = session_id else {
                        return false;
                    };

                    let Some(active) = st.active.remove(&session_id) else {
                        return false;
                    };

                    if let Some(target_id) = active.hovered_target {
                        st.delivered.insert(
                            target_id,
                            DeliveredColorDrop {
                                tick_id: up.tick_id,
                                payload: active.payload,
                            },
                        );
                    }

                    true
                })
                .unwrap_or(false);

            if host
                .drag(up.pointer_id)
                .is_some_and(|drag| drag.kind == kind && drag.source_window == action_cx.window)
            {
                host.cancel_drag(up.pointer_id);
            }

            if was_dragging || delivered_or_active {
                host.request_redraw(action_cx.window);
                return PressablePointerUpResult::SkipActivate;
            }

            PressablePointerUpResult::Continue
        })
    });
}

fn color_drag_kind_for_element(element: GlobalElementId) -> DragKindId {
    DragKindId(COLOR_DRAG_KIND_MASK ^ element.0)
}

fn color_drag_threshold_exceeded(start: Point, position: Point, threshold: Px) -> bool {
    let dx = position.x.0 - start.x.0;
    let dy = position.y.0 - start.y.0;
    let distance_sq = dx * dx + dy * dy;
    distance_sq >= threshold.0 * threshold.0
}
