use std::sync::Arc;

use fret_authoring::UiWriter;
use fret_runtime::KeyChord;
use fret_ui::UiHost;

use super::super::{ResponseExt, UiWriterImUiFacadeExt};
use crate::declarative::chrome::control_chrome_pressable_with_id_props;

mod behavior;
mod visual;

#[cfg(test)]
pub(super) use visual::combo_trigger_a11y_label;

pub(super) struct ComboTriggerOptions {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
    pub(super) open: bool,
}

pub(super) fn combo_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    preview: Arc<str>,
    options: ComboTriggerOptions,
) -> ResponseExt {
    ui.push_id(format!("{id}.trigger"), |ui| {
        let mut response = ResponseExt::default();

        let element = ui.with_cx_mut(|cx| {
            let response = &mut response;
            let props = visual::combo_trigger_props(visual::ComboTriggerPropsInput {
                enabled: options.enabled,
                focusable: options.focusable,
                a11y_label: options.a11y_label.clone(),
                test_id: options.test_id.clone(),
                open: options.open,
                label: label.clone(),
                preview: preview.clone(),
            });

            let enabled = options.enabled;
            let open = options.open;
            let activate_shortcut = options.activate_shortcut;
            let shortcut_repeat = options.shortcut_repeat;
            let label_for_visuals = label.clone();
            let preview_for_visuals = preview.clone();
            control_chrome_pressable_with_id_props(cx, move |cx, state, id| {
                behavior::install_combo_trigger_behavior(
                    cx,
                    id,
                    state,
                    behavior::ComboTriggerBehaviorInput {
                        enabled,
                        activate_shortcut,
                        shortcut_repeat,
                    },
                    response,
                );

                let (palette, chrome) = visual::combo_trigger_chrome(cx, enabled, state);

                (props, chrome, move |cx| {
                    visual::combo_trigger_children(
                        cx,
                        label_for_visuals,
                        preview_for_visuals,
                        open,
                        palette,
                    )
                })
            })
        });

        ui.add(element);
        response
    })
}
