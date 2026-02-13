use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_external_drag_region<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::ExternalDragRegionProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    let Event::ExternalDrag(e) = event else {
        return;
    };

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::ExternalDragActionHooks::default,
        |hooks| hooks.on_external_drag.clone(),
    );

    let Some(h) = hook else {
        return;
    };

    struct ExternalDragHookHost<'a, H: UiHost> {
        app: &'a mut H,
        notify_requested: &'a mut bool,
        notify_requested_location: &'a mut Option<crate::widget::UiSourceLocation>,
    }

    impl<H: UiHost> action::UiActionHost for ExternalDragHookHost<'_, H> {
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

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            self.app.next_share_sheet_token()
        }

        #[track_caller]
        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
            if self.notify_requested_location.is_none() {
                let caller = std::panic::Location::caller();
                *self.notify_requested_location = Some(crate::widget::UiSourceLocation {
                    file: caller.file(),
                    line: caller.line(),
                    column: caller.column(),
                });
            }
        }
    }

    let mut host = ExternalDragHookHost {
        app: &mut *cx.app,
        notify_requested: &mut cx.notify_requested,
        notify_requested_location: &mut cx.notify_requested_location,
    };
    let handled = h(
        &mut host,
        action::ActionCx {
            window,
            target: this.element,
        },
        e,
    );

    if handled {
        cx.stop_propagation();
    }
}
