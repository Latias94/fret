use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_text_input_region<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::TextInputRegionProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    struct TextInputRegionHookHost<'a, H: UiHost> {
        app: &'a mut H,
        notify_requested: &'a mut bool,
    }

    impl<H: UiHost> action::UiActionHost for TextInputRegionHookHost<'_, H> {
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

        fn notify(&mut self, _cx: action::ActionCx) {
            *self.notify_requested = true;
        }
    }

    let action_cx = action::ActionCx {
        window,
        target: this.element,
    };

    match event {
        Event::TextInput(text) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::TextInputRegionActionHooks::default,
                |hooks| hooks.on_text_input.clone(),
            );
            let Some(hook) = hook else {
                return;
            };
            let mut host = TextInputRegionHookHost {
                app: &mut *cx.app,
                notify_requested: &mut cx.notify_requested,
            };
            if hook(&mut host, action_cx, text.as_str()) {
                cx.stop_propagation();
            }
        }
        Event::Ime(ime) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::TextInputRegionActionHooks::default,
                |hooks| hooks.on_ime.clone(),
            );
            let Some(hook) = hook else {
                return;
            };
            let mut host = TextInputRegionHookHost {
                app: &mut *cx.app,
                notify_requested: &mut cx.notify_requested,
            };
            if hook(&mut host, action_cx, ime) {
                cx.stop_propagation();
            }
        }
        Event::ClipboardText { token, text } => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::TextInputRegionActionHooks::default,
                |hooks| hooks.on_clipboard_text.clone(),
            );
            let Some(hook) = hook else {
                return;
            };
            let mut host = TextInputRegionHookHost {
                app: &mut *cx.app,
                notify_requested: &mut cx.notify_requested,
            };
            if hook(&mut host, action_cx, *token, text.as_str()) {
                cx.stop_propagation();
            }
        }
        Event::ClipboardTextUnavailable { token } => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::TextInputRegionActionHooks::default,
                |hooks| hooks.on_clipboard_unavailable.clone(),
            );
            let Some(hook) = hook else {
                return;
            };
            let mut host = TextInputRegionHookHost {
                app: &mut *cx.app,
                notify_requested: &mut cx.notify_requested,
            };
            if hook(&mut host, action_cx, *token) {
                cx.stop_propagation();
            }
        }
        Event::SetTextSelection { anchor, focus } => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::TextInputRegionActionHooks::default,
                |hooks| hooks.on_set_selection.clone(),
            );
            let Some(hook) = hook else {
                return;
            };
            let mut host = TextInputRegionHookHost {
                app: &mut *cx.app,
                notify_requested: &mut cx.notify_requested,
            };
            if hook(&mut host, action_cx, *anchor, *focus) {
                cx.stop_propagation();
            }
        }
        _ => {}
    }
}
