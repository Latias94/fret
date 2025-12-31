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

    let Event::KeyDown { key, repeat, .. } = event else {
        return;
    };
    if *repeat {
        return;
    }

    enum Nav {
        Prev,
        Next,
        Home,
        End,
    }

    let nav = match (props.flex.direction, key) {
        (_, fret_core::KeyCode::Home) => Some(Nav::Home),
        (_, fret_core::KeyCode::End) => Some(Nav::End),
        (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowUp) => Some(Nav::Prev),
        (fret_core::Axis::Vertical, fret_core::KeyCode::ArrowDown) => Some(Nav::Next),
        (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowLeft) => Some(Nav::Prev),
        (fret_core::Axis::Horizontal, fret_core::KeyCode::ArrowRight) => Some(Nav::Next),
        _ => None,
    };
    let len = cx.children.len();
    if len == 0 {
        return;
    }

    let current = cx
        .focus
        .and_then(|focus| cx.children.iter().position(|n| *n == focus));

    let is_disabled =
        |idx: usize| -> bool { props.roving.disabled.get(idx).copied().unwrap_or(false) };

    let mut target: Option<usize> = None;
    match nav {
        Some(Nav::Home) => {
            target = (0..len).find(|&i| !is_disabled(i));
        }
        Some(Nav::End) => {
            target = (0..len).rev().find(|&i| !is_disabled(i));
        }
        Some(Nav::Next) if props.roving.wrap => {
            let Some(current) = current else {
                return;
            };
            for step in 1..=len {
                let idx = (current + step) % len;
                if !is_disabled(idx) {
                    target = Some(idx);
                    break;
                }
            }
        }
        Some(Nav::Prev) if props.roving.wrap => {
            let Some(current) = current else {
                return;
            };
            for step in 1..=len {
                let idx = (current + len - (step % len)) % len;
                if !is_disabled(idx) {
                    target = Some(idx);
                    break;
                }
            }
        }
        Some(Nav::Next) => {
            let Some(current) = current else {
                return;
            };
            target = ((current + 1)..len).find(|&i| !is_disabled(i));
        }
        Some(Nav::Prev) => {
            let Some(current) = current else {
                return;
            };
            if current > 0 {
                target = (0..current).rev().find(|&i| !is_disabled(i));
            }
        }
        None => {}
    }

    let key_to_ascii = |key: fret_core::KeyCode| -> Option<char> {
        use fret_core::KeyCode;
        Some(match key {
            KeyCode::KeyA => 'a',
            KeyCode::KeyB => 'b',
            KeyCode::KeyC => 'c',
            KeyCode::KeyD => 'd',
            KeyCode::KeyE => 'e',
            KeyCode::KeyF => 'f',
            KeyCode::KeyG => 'g',
            KeyCode::KeyH => 'h',
            KeyCode::KeyI => 'i',
            KeyCode::KeyJ => 'j',
            KeyCode::KeyK => 'k',
            KeyCode::KeyL => 'l',
            KeyCode::KeyM => 'm',
            KeyCode::KeyN => 'n',
            KeyCode::KeyO => 'o',
            KeyCode::KeyP => 'p',
            KeyCode::KeyQ => 'q',
            KeyCode::KeyR => 'r',
            KeyCode::KeyS => 's',
            KeyCode::KeyT => 't',
            KeyCode::KeyU => 'u',
            KeyCode::KeyV => 'v',
            KeyCode::KeyW => 'w',
            KeyCode::KeyX => 'x',
            KeyCode::KeyY => 'y',
            KeyCode::KeyZ => 'z',
            KeyCode::Digit0 => '0',
            KeyCode::Digit1 => '1',
            KeyCode::Digit2 => '2',
            KeyCode::Digit3 => '3',
            KeyCode::Digit4 => '4',
            KeyCode::Digit5 => '5',
            KeyCode::Digit6 => '6',
            KeyCode::Digit7 => '7',
            KeyCode::Digit8 => '8',
            KeyCode::Digit9 => '9',
            _ => return None,
        })
    };

    if target.is_none()
        && let Some(ch) = key_to_ascii(*key)
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

    let Some(target) = target else {
        return;
    };
    if current.is_some_and(|current| target == current) {
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
