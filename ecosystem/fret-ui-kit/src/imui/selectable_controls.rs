//! Immediate-mode selectable row helpers.

use std::sync::Arc;

use fret_ui::UiHost;

use super::label_identity::parse_label_identity;
use super::{ResponseExt, SelectableOptions, UiWriterImUiFacadeExt};

mod behavior;
mod keyboard;
mod props;
mod visual;

use visual::selectable_row_element;

pub(super) fn selectable_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let parts = parse_label_identity(label.as_ref());
    let identity = Arc::<str>::from(parts.identity);
    let visible_label = Arc::<str>::from(parts.visible);
    ui.push_id(("selectable-label", identity), |ui| {
        selectable_with_options_inner(ui, visible_label, options)
    })
}

fn selectable_with_options_inner<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SelectableOptions,
) -> ResponseExt {
    let mut response = ResponseExt::default();

    let element = ui.with_cx_mut(|cx| {
        let response = &mut response;
        let enabled = options.enabled && !super::imui_is_disabled(cx);
        let focusable = enabled && options.focusable;
        let selected = options.selected;
        let highlighted = enabled && options.highlighted;
        let close_popup = options.close_popup.clone();
        let activate_shortcut = options.activate_shortcut;
        let shortcut_repeat = options.shortcut_repeat;

        let props =
            props::selectable_pressable_props(&label, &options, enabled, focusable, selected);

        cx.pressable_with_id(props, move |cx, state, id| {
            behavior::install_selectable_behavior(
                cx,
                id,
                state,
                behavior::SelectableBehaviorOptions {
                    enabled,
                    focusable,
                    close_popup: close_popup.clone(),
                    activate_shortcut,
                    shortcut_repeat,
                },
                response,
            );

            vec![selectable_row_element(
                cx,
                label.clone(),
                enabled,
                selected,
                highlighted,
                state,
            )]
        })
    });

    ui.add(element);
    response
}

#[cfg(test)]
mod tests;
