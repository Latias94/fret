use super::ElementHostWidget;
use crate::declarative::frame::DismissibleLayerProps;
use crate::declarative::prelude::*;

pub(super) fn handle_dismissible_layer<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    props: DismissibleLayerProps,
    event: &Event,
) {
    if !props.enabled {
        return;
    }

    match event {
        Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            repeat: false,
            ..
        } => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_dismiss_request.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    DismissReason::Escape,
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Observer {
                return;
            }
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_dismiss_request.clone(),
            );

            if let Some(h) = hook {
                let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
                h(
                    &mut host,
                    action::ActionCx {
                        window,
                        target: this.element,
                    },
                    DismissReason::OutsidePress,
                );
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_type,
            ..
        }) => {
            let hook = crate::elements::with_element_state(
                &mut *cx.app,
                window,
                this.element,
                crate::action::DismissibleActionHooks::default,
                |hooks| hooks.on_pointer_move.clone(),
            );

            let Some(h) = hook else {
                return;
            };

            let mv = action::PointerMoveCx {
                position: *position,
                buttons: *buttons,
                modifiers: *modifiers,
                pointer_type: *pointer_type,
            };

            let mut host = action::UiActionHostAdapter { app: &mut *cx.app };
            let handled = h(
                &mut host,
                action::ActionCx {
                    window,
                    target: this.element,
                },
                mv,
            );

            if handled {
                cx.invalidate_self(Invalidation::Paint);
                cx.request_redraw();
            }
        }
        _ => {}
    }
}
