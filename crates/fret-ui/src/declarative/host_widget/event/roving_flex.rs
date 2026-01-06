use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_roving_flex<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: crate::element::RovingFlexProps,
    event: &Event,
) {
    if !props.roving.enabled {
        return;
    }

    let Event::KeyDown {
        key,
        modifiers,
        repeat,
    } = event
    else {
        return;
    };
    if *repeat {
        return;
    }
    let len = cx.children.len();
    if len == 0 {
        return;
    }

    let current = cx
        .focus
        .and_then(|focus| cx.children.iter().position(|n| *n == focus));

    let navigate_hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::RovingActionHooks::default,
        |hooks| hooks.on_navigate.clone(),
    );

    let mut target: Option<usize> = None;
    let mut handled = false;

    if let Some(h) = navigate_hook {
        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
        let result = h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            crate::action::RovingNavigateCx {
                key: *key,
                modifiers: *modifiers,
                repeat: *repeat,
                axis: props.flex.direction,
                current,
                len,
                disabled: props.roving.disabled.clone(),
                wrap: props.roving.wrap,
            },
        );

        if let crate::action::RovingNavigateResult::Handled {
            target: next_target,
        } = result
        {
            handled = true;
            target = next_target;
        }
    }

    if !handled
        && target.is_none()
        && let Some(ch) = fret_core::keycode_to_ascii_lowercase(*key)
    {
        let hook = crate::elements::with_element_state(
            &mut *cx.app,
            window,
            this.element,
            crate::action::RovingActionHooks::default,
            |hooks| hooks.on_typeahead.clone(),
        );

        if let Some(h) = hook {
            let tick = cx.app.tick_id().0;
            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
            target = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                crate::action::RovingTypeaheadCx {
                    input: ch,
                    current,
                    len,
                    disabled: props.roving.disabled.clone(),
                    wrap: props.roving.wrap,
                    tick,
                },
            );
        }
    }

    if handled && target.is_none() {
        cx.stop_propagation();
        return;
    }

    let Some(target) = target else {
        return;
    };
    if current.is_some_and(|current| target == current) {
        if handled {
            cx.stop_propagation();
        }
        return;
    }

    cx.request_focus(cx.children[target]);

    let hook = crate::elements::with_element_state(
        &mut *cx.app,
        window,
        this.element,
        crate::action::RovingActionHooks::default,
        |hooks| hooks.on_active_change.clone(),
    );

    if let Some(h) = hook {
        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
        h(
            &mut host,
            action::ActionCx {
                window,
                target: this.element,
            },
            target,
        );
    }

    cx.request_redraw();
    cx.stop_propagation();
}
