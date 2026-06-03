use std::sync::{Arc, Mutex};

use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::numeric_format::suppress_duplicate_chrome_affixes;
use crate::primitives::numeric_text_entry::NumericTextEntryFocusHandoffState;
use crate::primitives::style::EditorStyle;
use fret_ui::element::{AnyElement, Length, PressableA11y, PressableProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::Slider;
use super::chrome::{resolve_slider_geometry, resolve_slider_paint};
use super::frame::{SliderFrameArgs, slider_frame};
use super::model::{SliderMode, compose_affixed_value_text, hidden_layout};
use super::pointer::reset_slider_interaction;
use super::value_math::{quantize_value, t_from_value};

mod interaction;
mod typing_input;

use interaction::{SliderInteractionHandlersArgs, install_slider_interaction_handlers};
use typing_input::{SliderTypingInputArgs, slider_typing_input};

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

            install_slider_interaction_handlers(
                cx,
                SliderInteractionHandlersArgs {
                    state: state_for_slider.clone(),
                    focus_handoff: focus_handoff_for_slider.clone(),
                    model: model_for_change.clone(),
                    interactive_enabled,
                    allow_typing,
                    min,
                    max,
                    clamp,
                    step,
                    show_value,
                    value_width,
                    frame_padding_left: frame.padding.left,
                    frame_padding_right: frame.padding.right,
                    thumb_d,
                },
            );

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

    let input = slider_typing_input(
        cx,
        SliderTypingInputArgs {
            model: model.clone(),
            format: format.clone(),
            parse: parse.clone(),
            validate: validate.clone(),
            state: state.clone(),
            focus_handoff: focus_handoff.clone(),
            min,
            max,
            clamp,
            step,
            enabled,
            typing,
            input_layout,
            prefix: prefix.clone(),
            suffix: suffix.clone(),
            selection_behavior: options.selection_behavior,
            active_typing_test_id,
        },
    );

    cx.container(Default::default(), move |_cx| vec![slider_el, input])
}
