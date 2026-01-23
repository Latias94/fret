use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_internal_drag_region<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::InternalDragRegionProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    let Event::InternalDrag(e) = event else {
        return;
    };

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::InternalDragActionHooks::default,
        |hooks| hooks.on_internal_drag.clone(),
    );

    let Some(h) = hook else {
        return;
    };

    struct InternalDragHookHost<'a, H: UiHost> {
        app: &'a mut H,
        notify_requested: &'a mut bool,
    }

    impl<H: UiHost> action::UiActionHost for InternalDragHookHost<'_, H> {
        fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
            self.app.next_timer_token()
        }

        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
        }
    }

    impl<H: UiHost> action::UiDragActionHost for InternalDragHookHost<'_, H> {
        fn begin_drag_with_kind(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: fret_runtime::DragKindId,
            source_window: AppWindowId,
            start: Point,
        ) {
            fret_runtime::DragHost::begin_drag_with_kind(
                &mut *self.app,
                pointer_id,
                kind,
                source_window,
                start,
                (),
            );
        }

        fn begin_cross_window_drag_with_kind(
            &mut self,
            pointer_id: fret_core::PointerId,
            kind: fret_runtime::DragKindId,
            source_window: AppWindowId,
            start: Point,
        ) {
            fret_runtime::DragHost::begin_cross_window_drag_with_kind(
                &mut *self.app,
                pointer_id,
                kind,
                source_window,
                start,
                (),
            );
        }

        fn drag(&self, pointer_id: fret_core::PointerId) -> Option<&fret_runtime::DragSession> {
            fret_runtime::DragHost::drag(&*self.app, pointer_id)
        }

        fn drag_mut(
            &mut self,
            pointer_id: fret_core::PointerId,
        ) -> Option<&mut fret_runtime::DragSession> {
            fret_runtime::DragHost::drag_mut(&mut *self.app, pointer_id)
        }

        fn cancel_drag(&mut self, pointer_id: fret_core::PointerId) {
            fret_runtime::DragHost::cancel_drag(&mut *self.app, pointer_id);
        }
    }

    let internal = action::InternalDragCx {
        pointer_id: e.pointer_id,
        position: e.position,
        tick_id: cx.app.tick_id(),
        kind: e.kind,
        modifiers: e.modifiers,
    };

    let mut host = InternalDragHookHost {
        app: &mut *cx.app,
        notify_requested: &mut cx.notify_requested,
    };
    let handled = h(
        &mut host,
        action::ActionCx {
            window,
            target: this.element,
        },
        internal,
    );

    if handled {
        cx.stop_propagation();
    }
}
