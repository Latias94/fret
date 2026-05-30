use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::{Length, MainAlign, PressableProps};

use super::super::label_identity::parse_label_identity;
use super::super::{ResponseExt, SwitchOptions, UiWriterImUiFacadeExt};
use super::visual;
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod behavior;

pub(in crate::imui) fn switch_model_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: SwitchOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("switch-label", identity), |ui| {
        switch_model_with_options_inner(ui, visible_label, model, options)
    })
}

fn switch_model_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    model: &fret_runtime::Model<bool>,
    options: SwitchOptions,
) -> ResponseExt {
    let model = model.clone();
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        let value = cx
            .read_model(&model, fret_ui::Invalidation::Paint, |_app, v| *v)
            .unwrap_or(false);
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = enabled && options.focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height =
            Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = crate::primitives::switch::switch_a11y(
            options.a11y_label.clone().or_else(|| Some(label.clone())),
            value,
        );
        props.a11y.test_id = options.test_id.clone();

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            behavior::install_switch_behavior(
                cx,
                id,
                state,
                model.clone(),
                behavior::SwitchBehaviorOptions {
                    enabled,
                    focusable: options.focusable,
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            let (palette, chrome) = super::super::control_chrome::field_chrome(cx, enabled, state);
            let state_badge = visual::switch_state_badge(cx, palette, value);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                    move |cx| {
                        vec![
                            visual::boolean_label(cx, label_for_visuals.clone(), palette),
                            state_badge,
                        ]
                    },
                )]
            })
        })
    });

    ui.add(element);
    response
}
