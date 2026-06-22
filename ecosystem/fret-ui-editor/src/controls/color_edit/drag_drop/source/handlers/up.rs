use fret_core::{MouseButton, PointerId};
use fret_runtime::{DragKindId, Model, TickId};
use fret_ui::action::{PressablePointerUpResult, UiActionHost, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[cfg(test)]
use super::super::super::ActiveColorDrag;
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

        let delivered_or_active =
            finish_color_drag_for_source(host, &store, up.pointer_id, kind, source_id, up.tick_id);

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

fn finish_color_drag_for_source<H: UiActionHost + ?Sized>(
    host: &mut H,
    store: &Model<ColorDragDropStore>,
    pointer_id: PointerId,
    kind: DragKindId,
    source_id: GlobalElementId,
    tick_id: TickId,
) -> bool {
    let session_id = host
        .models_mut()
        .read(store, |st| {
            st.active.iter().find_map(|(session_id, active)| {
                (active.pointer_id == pointer_id
                    && active.kind == kind
                    && active.source_id == source_id)
                    .then_some(*session_id)
            })
        })
        .unwrap_or(None);

    let Some(session_id) = session_id else {
        return false;
    };

    host.update_model(store, |st| {
        let Some(active) = st.active.remove(&session_id) else {
            return false;
        };

        if let Some(target_id) = active.hovered_target {
            st.delivered.insert(
                target_id,
                DeliveredColorDrop {
                    tick_id,
                    payload: active.payload,
                },
            );
        }

        true
    })
    .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::Color;
    use fret_runtime::DragSessionId;
    use fret_ui::action::UiActionHostAdapter;

    use super::*;
    use crate::controls::color_edit::ColorEditDragDropPayload;

    fn active_drag(
        pointer_id: PointerId,
        kind: DragKindId,
        source_id: GlobalElementId,
        hovered_target: Option<GlobalElementId>,
    ) -> ActiveColorDrag {
        ActiveColorDrag {
            pointer_id,
            kind,
            source_id,
            hovered_target,
            payload: ColorEditDragDropPayload::from_color(
                Color::from_srgb_hex_rgb(0x33_66_99),
                false,
            ),
        }
    }

    #[test]
    fn pointer_up_empty_active_store_does_not_bump_revision() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        let revision = store.revision(&app);
        let finished = {
            let mut host = UiActionHostAdapter { app: &mut app };
            finish_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
                TickId(4),
            )
        };

        assert!(!finished);
        assert_eq!(store.revision(&app), revision);
    }

    #[test]
    fn pointer_up_unmatched_active_store_does_not_bump_revision() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        app.models_mut()
            .update(&store, |st| {
                st.active.insert(
                    DragSessionId(1),
                    active_drag(PointerId(9), DragKindId(2), GlobalElementId(3), None),
                );
            })
            .unwrap();
        let revision = store.revision(&app);
        let finished = {
            let mut host = UiActionHostAdapter { app: &mut app };
            finish_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
                TickId(4),
            )
        };

        assert!(!finished);
        assert_eq!(store.revision(&app), revision);
        assert_eq!(
            app.models_mut().read(&store, |st| st.active.len()).unwrap(),
            1
        );
    }

    #[test]
    fn pointer_up_matching_active_store_removes_and_delivers_entry() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        let target_id = GlobalElementId(9);
        let payload =
            ColorEditDragDropPayload::from_color(Color::from_srgb_hex_rgb(0x33_66_99), false);
        app.models_mut()
            .update(&store, |st| {
                let mut active = active_drag(
                    PointerId(1),
                    DragKindId(2),
                    GlobalElementId(3),
                    Some(target_id),
                );
                active.payload = payload;
                st.active.insert(DragSessionId(1), active);
            })
            .unwrap();
        let revision = store.revision(&app);
        let finished = {
            let mut host = UiActionHostAdapter { app: &mut app };
            finish_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
                TickId(4),
            )
        };

        assert!(finished);
        assert_ne!(store.revision(&app), revision);
        assert!(
            app.models_mut()
                .read(&store, |st| st.active.is_empty())
                .unwrap()
        );
        let delivered = app
            .models_mut()
            .read(&store, |st| st.delivered.get(&target_id).copied())
            .unwrap();
        assert_eq!(delivered.map(|drop| drop.tick_id), Some(TickId(4)));
        assert_eq!(delivered.map(|drop| drop.payload), Some(payload));
    }
}
