use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{PressablePointerUpResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ColorDragDropStore, DeliveredColorDrop};

pub(super) fn install_color_drag_pointer_up<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source_id: GlobalElementId,
    store: Model<ColorDragDropStore>,
    kind: DragKindId,
) {
    cx.pressable_add_on_pointer_up(std::sync::Arc::new(move |host, action_cx, up| {
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
    }));
}
