//! Immediate-mode slider helpers.

use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::element::{Length, PressableA11y, PressableProps};

use super::label_identity::parse_label_identity;
use super::{ResponseExt, SliderOptions, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod a11y;
mod interaction;
mod visual;

pub(super) fn slider_f32_model_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<f32>,
    options: SliderOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("slider-label", identity), |ui| {
        slider_f32_model_with_options_inner(ui, visible_label, model, options)
    })
}

fn slider_f32_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<f32>,
    options: SliderOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let min = options.min;
    let max = options.max;
    let step = options.step;

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height = Some(Length::Px(super::control_chrome::FIELD_MIN_HEIGHT));

        props.a11y = PressableA11y {
            role: Some(SemanticsRole::Slider),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let slider_a11y = a11y::resolve(cx, &model, min, max, step);
        let current = slider_a11y.current;
        let a11y_min = slider_a11y.min;
        let a11y_max = slider_a11y.max;
        let semantics = slider_a11y.decoration;

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            let active_item_model = interaction::install_slider_handlers(
                cx,
                id,
                enabled,
                model.clone(),
                min,
                max,
                step,
            );
            let progress = visual::slider_progress(current, a11y_min, a11y_max);

            let changed = cx.take_transient_for(id, super::KEY_CHANGED);
            let hover_delay =
                super::install_hover_query_hooks_for_pressable(cx, id, state.hovered_raw, None);
            super::populate_pressable_response(
                cx,
                id,
                state,
                hover_delay,
                &active_item_model,
                false,
                changed,
                state.pressed,
                changed,
                enabled,
                response,
            );

            let (palette, chrome) = super::control_chrome::field_chrome(cx, enabled, state);
            (props, chrome, move |cx| {
                visual::slider_children(cx, label_for_visuals, current, progress, palette)
            })
        })
        .attach_semantics(semantics)
    });

    ui.add(element);
    response
}
