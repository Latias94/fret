use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::UiHost;
use fret_ui::element::{Length, MainAlign, PressableA11y, PressableProps};

use super::super::label_identity::parse_label_identity;
use super::super::{RadioOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::visual;
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod behavior;

pub(in crate::imui) fn radio_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    selected: bool,
    options: RadioOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("radio-label", identity), |ui| {
        radio_with_options_inner(ui, visible_label, selected, options)
    })
}

fn radio_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    selected: bool,
    options: RadioOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let mut props = PressableProps::default();
        props.enabled = enabled;
        props.focusable = focusable;
        props.layout.size.width = Length::Fill;
        props.layout.size.min_height =
            Some(Length::Px(super::super::control_chrome::FIELD_MIN_HEIGHT));
        props.a11y = PressableA11y {
            role: Some(SemanticsRole::RadioButton),
            label: options.a11y_label.clone().or_else(|| Some(label.clone())),
            checked: Some(selected),
            test_id: options.test_id.clone(),
            ..Default::default()
        };

        let label_for_visuals = label.clone();
        control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
            behavior::install_radio_behavior(
                cx,
                id,
                state,
                behavior::RadioBehaviorOptions {
                    enabled,
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            let (palette, chrome) = super::super::control_chrome::field_chrome(cx, enabled, state);
            let indicator = visual::radio_indicator(cx, palette, selected);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::super::control_chrome::fill_row_props(MainAlign::Start),
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
