use std::any::Any;
use std::rc::Rc;
use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::{DragKindId, Model};
use fret_ui::action::PressablePointerDownResult;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::DragSourceOptions;
use super::super::store::ImUiDragDropStore;

mod payload_lifecycle;

pub(super) fn install_drag_source_hooks<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    kind: DragKindId,
    store: Model<ImUiDragDropStore>,
    payload: Rc<dyn Any>,
    options: &DragSourceOptions,
) {
    if !options.enabled {
        return;
    }

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
                if drag.kind != kind || drag.source_window != acx.window || drag.cross_window_hover
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

    payload_lifecycle::install_payload_lifecycle_hooks(cx, trigger_id, kind, store, payload);
}
