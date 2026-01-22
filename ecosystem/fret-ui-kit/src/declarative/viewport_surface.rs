//! Viewport surface helpers (Tier A embedding).
//!
//! This module provides a reusable declarative wrapper that:
//! - composites an app-owned `RenderTargetId` into the UI tree (ADR 0007),
//! - forwards pointer + wheel input as `Effect::ViewportInput` (ADR 0147),
//! - keeps mapping semantics consistent with `ViewportMapping` (contain/cover/stretch).

use std::sync::Arc;

use fret_core::{
    MouseButton, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputKind,
    ViewportMapping,
};
use fret_runtime::{Effect, Model};
use fret_ui::action::{OnPointerCancel, OnPointerDown, OnPointerMove, OnPointerUp, OnWheel};
use fret_ui::element::{AnyElement, Length, PointerRegionProps};
use fret_ui::{ElementContext, UiHost};

use super::controllable_state::use_controllable_model;

#[derive(Debug, Clone, Copy)]
pub struct ViewportSurfacePanelProps {
    pub target: RenderTargetId,
    pub target_px_size: (u32, u32),
    pub fit: ViewportFit,
    pub opacity: f32,
    /// When enabled, forwards pointer + wheel events as `Effect::ViewportInput`.
    pub forward_input: bool,
}

impl Default for ViewportSurfacePanelProps {
    fn default() -> Self {
        Self {
            target: RenderTargetId::default(),
            target_px_size: (1, 1),
            fit: ViewportFit::Stretch,
            opacity: 1.0,
            forward_input: true,
        }
    }
}

fn mapping_for(host_bounds: fret_core::Rect, props: ViewportSurfacePanelProps) -> ViewportMapping {
    ViewportMapping {
        content_rect: host_bounds,
        target_px_size: props.target_px_size,
        fit: props.fit,
    }
}

fn push_viewport_input(
    host: &mut dyn fret_ui::action::UiPointerActionHost,
    window: fret_core::AppWindowId,
    props: ViewportSurfacePanelProps,
    pixels_per_point: f32,
    pointer_id: fret_core::PointerId,
    pointer_type: fret_core::PointerType,
    position: fret_core::Point,
    kind: ViewportInputKind,
    clamped: bool,
) -> bool {
    let mapping = mapping_for(host.bounds(), props);
    let event = if clamped {
        ViewportInputEvent::from_mapping_window_point_clamped(
            window,
            props.target,
            &mapping,
            pixels_per_point,
            pointer_id,
            pointer_type,
            position,
            kind,
        )
    } else {
        let Some(event) = ViewportInputEvent::from_mapping_window_point(
            window,
            props.target,
            &mapping,
            pixels_per_point,
            pointer_id,
            pointer_type,
            position,
            kind,
        ) else {
            return false;
        };
        event
    };

    host.push_effect(Effect::ViewportInput(event));
    true
}

#[track_caller]
pub fn viewport_surface_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: ViewportSurfacePanelProps,
) -> AnyElement {
    let capture_button: Model<Option<MouseButton>> =
        use_controllable_model(cx, None::<Model<Option<MouseButton>>>, || None).model();
    let last_position: Model<Option<fret_core::Point>> =
        use_controllable_model(cx, None::<Model<Option<fret_core::Point>>>, || None).model();

    let props_c = props;
    let capture_button_c = capture_button.clone();
    let last_position_c = last_position.clone();
    let on_down: OnPointerDown = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              down: fret_ui::action::PointerDownCx| {
            if !props_c.forward_input {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&last_position_c, |p| *p = Some(down.position));
            let ok = push_viewport_input(
                host,
                action_cx.window,
                props_c,
                down.pixels_per_point,
                down.pointer_id,
                down.pointer_type,
                down.position,
                ViewportInputKind::PointerDown {
                    button: down.button,
                    modifiers: down.modifiers,
                    click_count: down.click_count,
                },
                false,
            );
            if !ok {
                return false;
            }

            host.capture_pointer();
            let _ = host
                .models_mut()
                .update(&capture_button_c, |b| *b = Some(down.button));
            host.request_redraw(action_cx.window);
            true
        },
    );

    let props_c = props;
    let capture_button_c = capture_button.clone();
    let last_position_c = last_position.clone();
    let on_move: OnPointerMove = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              mv: fret_ui::action::PointerMoveCx| {
            if !props_c.forward_input {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&last_position_c, |p| *p = Some(mv.position));
            let captured = host
                .models_mut()
                .read(&capture_button_c, |b| *b)
                .ok()
                .flatten()
                .is_some();

            push_viewport_input(
                host,
                action_cx.window,
                props_c,
                mv.pixels_per_point,
                mv.pointer_id,
                mv.pointer_type,
                mv.position,
                ViewportInputKind::PointerMove {
                    buttons: mv.buttons,
                    modifiers: mv.modifiers,
                },
                captured,
            )
        },
    );

    let props_c = props;
    let capture_button_c = capture_button.clone();
    let last_position_c = last_position.clone();
    let on_up: OnPointerUp = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              up: fret_ui::action::PointerUpCx| {
            if !props_c.forward_input {
                return false;
            }

            let _ = host
                .models_mut()
                .update(&last_position_c, |p| *p = Some(up.position));
            let captured_button = host
                .models_mut()
                .read(&capture_button_c, |b| *b)
                .ok()
                .flatten();

            let clamped = captured_button.is_some();
            let ok = push_viewport_input(
                host,
                action_cx.window,
                props_c,
                up.pixels_per_point,
                up.pointer_id,
                up.pointer_type,
                up.position,
                ViewportInputKind::PointerUp {
                    button: up.button,
                    modifiers: up.modifiers,
                    is_click: up.is_click,
                    click_count: up.click_count,
                },
                clamped,
            );

            if clamped {
                host.release_pointer_capture();
            }
            let _ = host.models_mut().update(&capture_button_c, |b| *b = None);
            if clamped {
                host.request_redraw(action_cx.window);
            }
            ok
        },
    );

    let props_c = props;
    let on_wheel: OnWheel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              wheel: fret_ui::action::WheelCx| {
            if !props_c.forward_input {
                return false;
            }

            push_viewport_input(
                host,
                action_cx.window,
                props_c,
                wheel.pixels_per_point,
                wheel.pointer_id,
                wheel.pointer_type,
                wheel.position,
                ViewportInputKind::Wheel {
                    delta: wheel.delta,
                    modifiers: wheel.modifiers,
                },
                false,
            )
        },
    );

    let props_c = props;
    let capture_button_c = capture_button.clone();
    let last_position_c = last_position.clone();
    let on_cancel: OnPointerCancel = Arc::new(
        move |host: &mut dyn fret_ui::action::UiPointerActionHost,
              action_cx: fret_ui::action::ActionCx,
              cancel: fret_ui::action::PointerCancelCx| {
            if !props_c.forward_input {
                return false;
            }

            let position = cancel
                .position
                .or_else(|| {
                    host.models_mut()
                        .read(&last_position_c, |p| *p)
                        .ok()
                        .flatten()
                })
                .unwrap_or_else(|| host.bounds().origin);

            let captured = host
                .models_mut()
                .read(&capture_button_c, |b| *b)
                .ok()
                .flatten()
                .is_some();

            let ok = push_viewport_input(
                host,
                action_cx.window,
                props_c,
                cancel.pixels_per_point,
                cancel.pointer_id,
                cancel.pointer_type,
                position,
                ViewportInputKind::PointerCancel {
                    buttons: cancel.buttons,
                    modifiers: cancel.modifiers,
                    reason: cancel.reason,
                },
                captured,
            );

            let _ = host.models_mut().update(&capture_button_c, |b| *b = None);
            let _ = host.models_mut().update(&last_position_c, |p| *p = None);
            if captured {
                host.release_pointer_capture();
                host.request_redraw(action_cx.window);
            }

            ok
        },
    );

    let mut region = PointerRegionProps::default();
    region.layout.size.width = Length::Fill;
    region.layout.size.height = Length::Fill;

    cx.pointer_region(region, |cx| {
        cx.pointer_region_on_pointer_down(on_down);
        cx.pointer_region_on_pointer_move(on_move);
        cx.pointer_region_on_pointer_up(on_up);
        cx.pointer_region_on_pointer_cancel(on_cancel);
        cx.pointer_region_on_wheel(on_wheel);

        let mut props2 = fret_ui::element::ViewportSurfaceProps::new(props.target);
        props2.target_px_size = props.target_px_size;
        props2.fit = props.fit;
        props2.opacity = props.opacity;
        vec![cx.viewport_surface_props(props2)]
    })
}
