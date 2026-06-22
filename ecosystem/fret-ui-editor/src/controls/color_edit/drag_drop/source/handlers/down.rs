use fret_core::{MouseButton, PointerId};
use fret_runtime::{DragKindId, Model};
use fret_ui::action::{PressablePointerDownResult, UiActionHost, UiActionHostExt as _};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

#[cfg(test)]
use super::super::super::ActiveColorDrag;
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

        remove_existing_color_drag_for_source(host, &store, down.pointer_id, kind, source_id);

        PressablePointerDownResult::Continue
    }));
}

fn remove_existing_color_drag_for_source<H: UiActionHost + ?Sized>(
    host: &mut H,
    store: &Model<ColorDragDropStore>,
    pointer_id: PointerId,
    kind: DragKindId,
    source_id: GlobalElementId,
) -> bool {
    let has_matching_active = host
        .models_mut()
        .read(store, |st| {
            st.active.values().any(|active| {
                active.pointer_id == pointer_id
                    && active.kind == kind
                    && active.source_id == source_id
            })
        })
        .unwrap_or(false);

    if !has_matching_active {
        return false;
    }

    host.update_model(store, |st| {
        let before = st.active.len();
        st.active.retain(|_, active| {
            !(active.pointer_id == pointer_id
                && active.kind == kind
                && active.source_id == source_id)
        });
        st.active.len() != before
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
    ) -> ActiveColorDrag {
        ActiveColorDrag {
            pointer_id,
            kind,
            source_id,
            hovered_target: None,
            payload: ColorEditDragDropPayload::from_color(
                Color::from_srgb_hex_rgb(0x33_66_99),
                false,
            ),
        }
    }

    #[test]
    fn pointer_down_empty_active_store_does_not_bump_revision() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        let revision = store.revision(&app);
        let removed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            remove_existing_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
            )
        };

        assert!(!removed);
        assert_eq!(store.revision(&app), revision);
    }

    #[test]
    fn pointer_down_unmatched_active_store_does_not_bump_revision() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        app.models_mut()
            .update(&store, |st| {
                st.active.insert(
                    DragSessionId(1),
                    active_drag(PointerId(9), DragKindId(2), GlobalElementId(3)),
                );
            })
            .unwrap();
        let revision = store.revision(&app);
        let removed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            remove_existing_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
            )
        };

        assert!(!removed);
        assert_eq!(store.revision(&app), revision);
        assert_eq!(
            app.models_mut().read(&store, |st| st.active.len()).unwrap(),
            1
        );
    }

    #[test]
    fn pointer_down_matching_active_store_removes_entry() {
        let mut app = App::new();
        let store = app.models_mut().insert(ColorDragDropStore::default());
        app.models_mut()
            .update(&store, |st| {
                st.active.insert(
                    DragSessionId(1),
                    active_drag(PointerId(1), DragKindId(2), GlobalElementId(3)),
                );
                st.active.insert(
                    DragSessionId(2),
                    active_drag(PointerId(9), DragKindId(2), GlobalElementId(3)),
                );
            })
            .unwrap();
        let revision = store.revision(&app);
        let removed = {
            let mut host = UiActionHostAdapter { app: &mut app };
            remove_existing_color_drag_for_source(
                &mut host,
                &store,
                PointerId(1),
                DragKindId(2),
                GlobalElementId(3),
            )
        };

        assert!(removed);
        assert_ne!(store.revision(&app), revision);
        assert_eq!(
            app.models_mut().read(&store, |st| st.active.len()).unwrap(),
            1
        );
        assert!(
            app.models_mut()
                .read(&store, |st| st
                    .active
                    .values()
                    .all(|active| active.pointer_id != PointerId(1)))
                .unwrap()
        );
    }
}
