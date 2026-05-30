use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, MouseButton, Point, PointerId, Px};
use fret_runtime::{DragKindId, DragPhase, DragSessionId, Model, TickId};
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, Invalidation, Theme, UiHost};

use super::model::format_hex;
use super::{
    ColorEditDragDropComponents, ColorEditDragDropOptions, ColorEditDragDropPayload,
    ColorEditPaletteEntry,
};

const COLOR_DRAG_KIND_MASK: u64 = 0x4000_0000_0000_0000;
const DEFAULT_COLOR_DRAG_THRESHOLD_PX: f32 = 6.0;

#[derive(Default)]
struct ColorDragDropStoreGlobal {
    model: Option<Model<ColorDragDropStore>>,
}

#[derive(Default)]
pub(in crate::controls::color_edit) struct ColorDragDropStore {
    active: HashMap<DragSessionId, ActiveColorDrag>,
    delivered: HashMap<GlobalElementId, DeliveredColorDrop>,
}

#[derive(Clone, Copy)]
struct ActiveColorDrag {
    pointer_id: PointerId,
    kind: DragKindId,
    source_id: GlobalElementId,
    hovered_target: Option<GlobalElementId>,
    payload: ColorEditDragDropPayload,
}

#[derive(Clone, Copy)]
struct DeliveredColorDrop {
    tick_id: TickId,
    payload: ColorEditDragDropPayload,
}

pub(super) fn color_drag_drop_store_for<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> Model<ColorDragDropStore> {
    cx.app
        .with_global_mut_untracked(ColorDragDropStoreGlobal::default, |st, app| {
            if let Some(model) = st.model.clone() {
                return model;
            }

            let model = app.models_mut().insert(ColorDragDropStore::default());
            st.model = Some(model.clone());
            model
        })
}

pub(super) fn prune_color_drag_drop_store<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
) {
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
            .retain(|_, drop| current_tick.0 <= drop.tick_id.0.saturating_add(1));
    });
}

pub(super) fn resolve_color_drag_threshold<H: UiHost>(cx: &ElementContext<'_, H>) -> Px {
    let threshold = Theme::global(&*cx.app)
        .metric_by_key(fret_ui_kit::theme_tokens::metric::COMPONENT_IMUI_DRAG_THRESHOLD_PX)
        .unwrap_or(Px(DEFAULT_COLOR_DRAG_THRESHOLD_PX));
    if threshold.0.is_finite() {
        Px(threshold.0.max(0.0))
    } else {
        Px(DEFAULT_COLOR_DRAG_THRESHOLD_PX)
    }
}

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
                Active(DragSessionId),
                Canceled(DragSessionId),
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

pub(in crate::controls::color_edit) fn update_color_drop_target<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
    target_id: GlobalElementId,
    hovered: bool,
    enabled: bool,
) -> bool {
    if !enabled {
        return false;
    }

    let active_sessions = cx
        .read_model(store, Invalidation::Paint, |app, st| {
            st.active
                .iter()
                .filter_map(|(session_id, active)| {
                    let drag = app.drag(active.pointer_id)?;
                    (drag.session_id == *session_id && drag.kind == active.kind && drag.dragging)
                        .then_some(*session_id)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if active_sessions.is_empty() {
        return false;
    }

    let mut over = false;
    let _ = cx.app.models_mut().update(store, |st| {
        for session_id in active_sessions {
            let Some(active) = st.active.get_mut(&session_id) else {
                continue;
            };
            if hovered {
                active.hovered_target = Some(target_id);
                over = true;
            } else if active.hovered_target == Some(target_id) {
                active.hovered_target = None;
            }
        }
    });
    over
}

pub(in crate::controls::color_edit) fn take_delivered_color_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    store: &Model<ColorDragDropStore>,
    target_id: GlobalElementId,
) -> Option<ColorEditDragDropPayload> {
    let current_tick = cx.app.tick_id();
    cx.app
        .models_mut()
        .update(store, |st| {
            let delivered = st.delivered.remove(&target_id)?;
            if current_tick.0 > delivered.tick_id.0.saturating_add(1) {
                return None;
            }
            Some(delivered.payload)
        })
        .ok()
        .flatten()
}

pub(super) struct ColorEditDeliveredDropArgs {
    pub(super) store: Model<ColorDragDropStore>,
    pub(super) target_id: GlobalElementId,
    pub(super) model: Model<Color>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) current: Color,
    pub(super) show_alpha: bool,
    pub(super) enabled: bool,
}

pub(super) fn apply_delivered_color_drop<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditDeliveredDropArgs,
) {
    if !args.enabled {
        return;
    }
    let Some(payload) = take_delivered_color_drop(cx, &args.store, args.target_id) else {
        return;
    };

    let current_for_drop = cx
        .get_model_copied(&args.model, Invalidation::Paint)
        .unwrap_or(args.current);
    let next = apply_color_drop_payload(payload, current_for_drop, args.show_alpha);
    let formatted = format_hex(next, args.show_alpha);
    let _ = cx
        .app
        .models_mut()
        .update(&args.model, |color| *color = next);
    let _ = cx
        .app
        .models_mut()
        .update(&args.draft, |s| *s = formatted.as_ref().to_string());
    let _ = cx.app.models_mut().update(&args.error, |e| *e = None);
}

pub(super) fn apply_color_drop_payload(
    payload: ColorEditDragDropPayload,
    current: Color,
    target_show_alpha: bool,
) -> Color {
    let mut next = payload.color();
    if payload.components() == ColorEditDragDropComponents::Rgb || !target_show_alpha {
        next.a = current.a;
    }
    next
}

pub(super) fn palette_slot_drop_from_payload(
    previous: ColorEditPaletteEntry,
    payload: ColorEditDragDropPayload,
) -> ColorEditPaletteEntry {
    previous.with_rgb(fret_ui_kit::colors::hex_rgb_from_linear(payload.color()))
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
