use fret_core::Px;
use fret_runtime::{DragKindId, DragPhase, DragSessionId, Model};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{ActiveColorDrag, ColorDragDropStore};
use super::color_drag_threshold_exceeded;
use crate::controls::color_edit::ColorEditDragDropPayload;

enum MoveResult {
    Pending,
    Active(DragSessionId),
    Canceled(DragSessionId),
}

pub(super) fn install_color_drag_pointer_move<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source_id: GlobalElementId,
    store: Model<ColorDragDropStore>,
    payload: ColorEditDragDropPayload,
    threshold: Px,
    kind: DragKindId,
) {
    cx.pressable_add_on_pointer_move(std::sync::Arc::new(move |host, action_cx, mv| {
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
            } else if color_drag_threshold_exceeded(drag.start_position, drag.position, threshold) {
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
    }));
}
