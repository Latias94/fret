use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{PressablePointerDownResult, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::ColorDragDropStore;
use crate::controls::color_edit::ColorEditDragDropOptions;

pub(super) fn install_color_drag_pointer_down<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    source_id: GlobalElementId,
    store: Model<ColorDragDropStore>,
    options: ColorEditDragDropOptions,
    kind: DragKindId,
) {
    cx.pressable_add_on_pointer_down(std::sync::Arc::new(move |host, action_cx, down| {
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
                host.begin_drag_with_kind(down.pointer_id, kind, action_cx.window, down.position);
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
    }));
}
