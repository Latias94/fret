use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_pressable<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: PressableProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }
    match event {
        Event::Pointer(pe) => match pe {
            fret_core::PointerEvent::Move { .. } => {
                cx.set_cursor_icon(CursorIcon::Pointer);
            }
            fret_core::PointerEvent::Down { button, .. } => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.request_focus(cx.node);
                cx.capture_pointer(cx.node);
                crate::elements::set_pressed_pressable(&mut *cx.app, window, Some(this.element));
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            fret_core::PointerEvent::Up {
                button, position, ..
            } => {
                if *button != MouseButton::Left {
                    return;
                }
                cx.release_pointer_capture();
                crate::elements::set_pressed_pressable(&mut *cx.app, window, None);

                // Activate based on the pointer-up position, not the cached hovered state. This
                // keeps click-through outside-press dismissal semantics (ADR 0069) robust even
                // when overlay policies update hover state in an observer pass.
                let hovered = cx.bounds.contains(*position);

                if hovered {
                    let hook = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        this.element,
                        crate::action::PressableActionHooks::default,
                        |hooks| hooks.on_activate.clone(),
                    );

                    if let Some(h) = hook {
                        let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                        h(
                            &mut host,
                            action::ActionCx {
                                window,
                                target: this.element,
                            },
                            ActivateReason::Pointer,
                        );
                    }
                }
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
            _ => {}
        },
        Event::KeyDown { key, repeat, .. } => {
            if *repeat {
                return;
            }
            if cx.focus != Some(cx.node) {
                return;
            }
            if !matches!(
                key,
                fret_core::KeyCode::Enter
                    | fret_core::KeyCode::NumpadEnter
                    | fret_core::KeyCode::Space
            ) {
                return;
            }
            crate::elements::set_pressed_pressable(&mut *cx.app, window, Some(this.element));
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        Event::KeyUp { key, .. } => {
            if cx.focus != Some(cx.node) {
                return;
            }
            if !matches!(
                key,
                fret_core::KeyCode::Enter
                    | fret_core::KeyCode::NumpadEnter
                    | fret_core::KeyCode::Space
            ) {
                return;
            }
            let pressed = crate::elements::is_pressed_pressable(&mut *cx.app, window, this.element);
            if !pressed {
                return;
            }
            crate::elements::set_pressed_pressable(&mut *cx.app, window, None);
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::PressableActionHooks::default,
                |hooks| hooks.on_activate.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    ActivateReason::Keyboard,
                );
            }
            cx.invalidate_self(Invalidation::Paint);
            cx.request_redraw();
            cx.stop_propagation();
        }
        _ => {}
    };
}
