use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::MainAlign;

use super::super::super::super::{
    CheckboxOptions, ResponseExt, UiWriterImUiFacadeExt, control_chrome, imui_is_disabled,
};
use super::super::super::visual;
use super::super::{behavior, props};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

pub(super) fn checkbox_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: CheckboxOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let props =
            props::checkbox_pressable_props(label.clone(), value, &options, enabled, focusable);

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            behavior::install_checkbox_behavior(
                cx,
                id,
                state,
                model.clone(),
                behavior::CheckboxBehaviorOptions {
                    enabled,
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            let (palette, chrome) = control_chrome::field_chrome(cx, enabled, state);
            let indicator = visual::checkbox_indicator(cx, palette, value);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    control_chrome::fill_row_props(MainAlign::Start),
                    move |cx| {
                        vec![
                            indicator,
                            visual::boolean_label(cx, label_for_visuals.clone(), palette),
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}
