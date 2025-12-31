use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_timer_event<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    event: &Event,
) -> bool {
    let Event::Timer { token } = event else {
        return false;
    };

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::TimerActionHooks::default,
        |hooks| hooks.on_timer.clone(),
    );

    if let Some(h) = hook {
        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
        let handled = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            *token,
        );
        if handled {
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
            return true;
        }
    }

    false
}

pub(super) fn try_key_hook<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    key: fret_core::KeyCode,
    modifiers: fret_core::Modifiers,
    repeat: bool,
) -> bool {
    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::KeyActionHooks::default,
        |hooks| hooks.on_key_down.clone(),
    );

    if let Some(h) = hook {
        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
        let handled = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            KeyDownCx {
                key,
                modifiers,
                repeat,
            },
        );
        if handled {
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
            return true;
        }
    }

    false
}
