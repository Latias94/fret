use std::sync::Arc;

use fret_ui::UiHost;
use fret_ui::element::MainAlign;

use super::super::super::label_identity::parse_label_identity;
use super::super::super::{RadioOptions, ResponseExt, UiWriterImUiFacadeExt};
use super::super::visual;
use super::{behavior, props};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

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
        let enabled = options.enabled && !super::super::super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let props =
            props::radio_pressable_props(label.clone(), selected, &options, enabled, focusable);

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

            let (palette, chrome) =
                super::super::super::control_chrome::field_chrome(cx, enabled, state);
            let indicator = visual::radio_indicator(cx, palette, selected);

            (props, chrome, move |cx| {
                vec![cx.flex(
                    super::super::super::control_chrome::fill_row_props(MainAlign::Start),
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
