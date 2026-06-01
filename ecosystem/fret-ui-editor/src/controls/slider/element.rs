use std::sync::{Arc, Mutex};

use crate::controls::numeric_input::{
    NumericInput, NumericInputErrorDisplay, NumericInputOptions, NumericInputOutcome,
};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::{
    NumericTextEntryFocusHandoffState, arm_numeric_text_entry_focus_handoff,
    sync_numeric_text_entry_focus_handoff,
};
use crate::primitives::style::EditorStyle;
use fret_core::{CursorIcon, MouseButton};
use fret_ui::action::{PressablePointerDownResult, PressablePointerUpResult};
use fret_ui::element::{AnyElement, Length, PressableA11y, PressableProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::Slider;
use super::chrome::{resolve_slider_geometry, resolve_slider_paint};
use super::frame::{SliderFrameArgs, slider_frame};
use super::model::{SliderMode, compose_affixed_value_text, hidden_layout};
use super::pointer::{
    begin_slider_drag, clear_slider_drag, enter_slider_typing, finish_slider_drag,
    is_slider_drag_pointer, reset_slider_interaction,
};
use super::typing::{slider_typing_parse, slider_typing_validate};
use super::value_math::{quantize_value, t_from_value, value_from_slider_local_x};

pub(super) fn slider_into_element_keyed<T, H>(
    slider: Slider<T>,
    cx: &mut ElementContext<'_, H>,
) -> AnyElement
where
    T: DragValueScalar + Default,
    H: UiHost,
{
    let Slider {
        model,
        min: raw_min,
        max: raw_max,
        format,
        parse,
        validate,
        options,
    } = slider;

    let theme = Theme::global(&*cx.app);
    let style = EditorStyle::resolve(theme);
    let density = style.density;
    let frame = style.frame_chrome_small();

    let geometry = resolve_slider_geometry(theme);
    let thumb_d = geometry.thumb_d;

    let (min, max) = if raw_min <= raw_max {
        (raw_min, raw_max)
    } else {
        (raw_max, raw_min)
    };

    let clamp = options.clamp;
    let step = options.step;

    let raw_value = cx
        .get_model_copied(&model, Invalidation::Paint)
        .unwrap_or_default();
    let value_f = raw_value.to_f64();
    let value_f = quantize_value(min, max, clamp, step, value_f);
    let t = t_from_value(min, max, clamp, value_f);

    // Anchor state to a stable element id under the slider's identity key. This avoids any
    // accidental state sharing across sibling sliders when the surrounding composition changes.
    let state_id = cx.named("slider.state", |cx| cx.root_id());
    let state: Arc<Mutex<super::model::SliderState>> = cx.state_for(
        state_id,
        || Arc::new(Mutex::new(super::model::SliderState::default())),
        |s| s.clone(),
    );
    let focus_handoff_state_id = cx.named("slider.focus_handoff.state", |cx| cx.root_id());
    let focus_handoff: Arc<Mutex<NumericTextEntryFocusHandoffState>> = cx.state_for(
        focus_handoff_state_id,
        || Arc::new(Mutex::new(NumericTextEntryFocusHandoffState::default())),
        |s| s.clone(),
    );

    let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;
    let typing = mode == SliderMode::Typing;

    let enabled = options.enabled;
    if !enabled && typing {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        reset_slider_interaction(&mut st);
    }

    let mut slider_layout = options.layout;
    if typing {
        slider_layout = hidden_layout(slider_layout);
    }

    let mut input_layout = options.layout;
    if !typing {
        input_layout = hidden_layout(input_layout);
    }

    let model_for_change = model.clone();
    let a11y_label = options.a11y_label.clone();
    let show_value = options.show_value;
    let value_width = options.value_width;
    let allow_typing = options.allow_typing;
    let typing_test_id = derived_test_id(options.test_id.as_ref(), "typing");
    let active_typing_test_id = if typing { typing_test_id.clone() } else { None };
    let value_display_test_id = derived_test_id(options.test_id.as_ref(), "value_display");

    let interactive_enabled = enabled && !typing;

    let mut layout = slider_layout;
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(density.row_height));
    }

    let display_format = format.clone();
    let value_text = (display_format)(T::from_f64(value_f));
    let (prefix, suffix) = suppress_duplicate_chrome_affixes(
        value_text.as_ref(),
        options.prefix.clone(),
        options.suffix.clone(),
    );
    let value_display_text =
        compose_affixed_value_text(&value_text, prefix.as_ref(), suffix.as_ref());

    let state_for_slider = state.clone();
    let focus_handoff_for_slider = focus_handoff.clone();
    let mut slider_el = cx.pressable(
        PressableProps {
            enabled: interactive_enabled,
            layout,
            a11y: PressableA11y {
                label: a11y_label,
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let slider_id = cx.root_id();
            {
                let mut st = state_for_slider.lock().unwrap_or_else(|e| e.into_inner());
                st.slider_id = Some(slider_id);
            }

            let state_for_down = state_for_slider.clone();
            let focus_handoff_for_down = focus_handoff_for_slider.clone();
            let model_for_down = model_for_change.clone();
            cx.pressable_add_on_pointer_down(Arc::new(move |host, action_cx, down| {
                if !interactive_enabled {
                    return PressablePointerDownResult::Continue;
                }
                if allow_typing && down.button == MouseButton::Left && down.click_count >= 2 {
                    let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                    enter_slider_typing(&mut st);
                    {
                        let mut handoff = focus_handoff_for_down
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        arm_numeric_text_entry_focus_handoff(&mut handoff);
                    }
                    host.request_redraw(action_cx.window);
                    return PressablePointerDownResult::SkipDefaultAndStopPropagation;
                }

                if down.button != MouseButton::Left {
                    return PressablePointerDownResult::Continue;
                }

                let bounds = host.bounds();
                let next = value_from_slider_local_x(
                    min,
                    max,
                    clamp,
                    step,
                    down.position_local.x.0 as f64,
                    bounds.size.width.0 as f64,
                    show_value,
                    value_width.0 as f64,
                    frame.padding.left.0 as f64,
                    frame.padding.right.0 as f64,
                    thumb_d.0 as f64,
                );
                let next_t = T::from_f64(next);
                let _ = host.models_mut().update(&model_for_down, |v| *v = next_t);
                host.request_redraw(action_cx.window);

                host.set_cursor_icon(CursorIcon::ColResize);

                let mut st = state_for_down.lock().unwrap_or_else(|e| e.into_inner());
                begin_slider_drag(&mut st, down.pointer_id);

                PressablePointerDownResult::Continue
            }));

            let state_for_move = state_for_slider.clone();
            let model_for_move = model_for_change.clone();
            cx.pressable_add_on_pointer_move(Arc::new(move |host, action_cx, mv| {
                let mut st_lock = state_for_move.lock().unwrap_or_else(|e| e.into_inner());
                if !is_slider_drag_pointer(&st_lock, mv.pointer_id) {
                    return false;
                }

                // Best-effort cleanup when the pointer-up event is missed.
                if !mv.buttons.left {
                    clear_slider_drag(&mut st_lock);
                    return false;
                }

                let bounds = host.bounds();
                let next = value_from_slider_local_x(
                    min,
                    max,
                    clamp,
                    step,
                    mv.position_local.x.0 as f64,
                    bounds.size.width.0 as f64,
                    show_value,
                    value_width.0 as f64,
                    frame.padding.left.0 as f64,
                    frame.padding.right.0 as f64,
                    thumb_d.0 as f64,
                );
                let next_t = T::from_f64(next);
                let _ = host.models_mut().update(&model_for_move, |v| *v = next_t);
                host.request_redraw(action_cx.window);
                true
            }));

            let state_for_up = state_for_slider.clone();
            cx.pressable_add_on_pointer_up(Arc::new(move |_host, _action_cx, up| {
                let mut st = state_for_up.lock().unwrap_or_else(|e| e.into_inner());
                finish_slider_drag(&mut st, up.pointer_id);
                PressablePointerUpResult::Continue
            }));

            let theme = Theme::global(&*cx.app);
            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;
            let focused = st.focused;

            let paint = resolve_slider_paint(theme, interactive_enabled, enabled, hovered, pressed);

            vec![slider_frame(
                cx,
                SliderFrameArgs {
                    density,
                    frame_chrome: frame,
                    geometry,
                    paint,
                    t,
                    interactive_enabled,
                    hovered,
                    pressed,
                    focused,
                    show_value,
                    value_width,
                    value_display_text: value_display_text.clone(),
                    value_display_test_id: value_display_test_id.clone(),
                },
            )]
        },
    );

    if let Some(test_id) = options.test_id.as_ref() {
        slider_el = slider_el.test_id(test_id.clone());
    }

    let parse_for_input = slider_typing_parse(parse.clone(), min, max, clamp, step);
    let validate_for_input = slider_typing_validate(validate.clone(), min, max, clamp);

    let state_for_input = state.clone();
    let input_focus_target: Arc<Mutex<Option<fret_ui::GlobalElementId>>> =
        Arc::new(Mutex::new(None));
    let input = NumericInput::new(model.clone(), format.clone(), parse_for_input)
        .validate(validate_for_input)
        .focus_target(input_focus_target.clone())
        .options(NumericInputOptions {
            layout: input_layout,
            enabled: enabled && typing,
            focusable: enabled && typing,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            selection_behavior: options.selection_behavior,
            test_id: active_typing_test_id,
            // Avoid growing the row height when a commit-time validation error occurs.
            // A small trailing status icon keeps the inspector layout stable.
            error_display: NumericInputErrorDisplay::TrailingIcon,
            ..Default::default()
        })
        .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
            if matches!(
                outcome,
                NumericInputOutcome::Committed | NumericInputOutcome::Canceled
            ) {
                let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                reset_slider_interaction(&mut st);
                if let Some(slider_id) = st.slider_id {
                    host.request_focus(slider_id);
                }
                host.request_redraw(action_cx.window);
            }
        })))
        .into_element(cx);

    if let Some(input_id) = input_focus_target
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .copied()
    {
        let is_focused = cx.is_focused_element(input_id);
        sync_numeric_text_entry_focus_handoff(
            cx,
            input.id,
            &focus_handoff,
            typing,
            input_id,
            is_focused,
        );
    }

    cx.container(Default::default(), move |_cx| vec![slider_el, input])
}
